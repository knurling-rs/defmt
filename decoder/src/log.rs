//! This module provides interoperability utilities between [`defmt`] and the [`log`] crate.
//!
//! If you are implementing a custom defmt decoding tool, this module can make it easier to
//! integrate it with logs produced with the [`log`] crate.
//!
//! [`log`]: https://crates.io/crates/log
//! [`defmt`]: https://crates.io/crates/defmt

use colored::{Color, Colorize};
use difference::{Changeset, Difference};
use log::{Level, Log, Metadata, Record};
use serde_json::{json, Value as JsonValue};

use std::{
    fmt::{self, Write as _},
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::Frame;

const DEFMT_TARGET_MARKER: &str = "defmt@";

/// Logs a defmt frame using the `log` facade.
pub fn log_defmt(
    frame: &Frame<'_>,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) {
    let timestamp = frame.display_timestamp().map(|display| display.to_string());
    let display = frame.display_message();

    if let Some(level) = frame.level() {
        let level = match level {
            crate::Level::Trace => Level::Trace,
            crate::Level::Debug => Level::Debug,
            crate::Level::Info => Level::Info,
            crate::Level::Warn => Level::Warn,
            crate::Level::Error => Level::Error,
        };

        let target = format!("{}{}", DEFMT_TARGET_MARKER, timestamp.unwrap_or_default());
        log::logger().log(
            &Record::builder()
                .args(format_args!("{}", display))
                .level(level)
                .target(&target)
                .module_path(module_path)
                .file(file)
                .line(line)
                .build(),
        );
    } else {
        let stdout = io::stdout();
        let mut sink = stdout.lock();
        let timestamp = timestamp.map(|ts| format!("{} ", ts)).unwrap_or_default();
        writeln!(&mut sink, "{}{}", timestamp, display).ok();
        print_location(&mut sink, file, line, module_path).ok();
    }
}

/// Determines whether `metadata` belongs to a log record produced by [`log_defmt`].
pub fn is_defmt_frame(metadata: &Metadata) -> bool {
    metadata.target().starts_with(DEFMT_TARGET_MARKER)
}

/// A `log` record representing a defmt log frame.
pub struct DefmtRecord<'a> {
    timestamp: &'a str,
    log_record: &'a Record<'a>,
}

impl<'a> DefmtRecord<'a> {
    /// If `record` was produced by [`log_defmt`], returns the corresponding `DefmtRecord`.
    pub fn new(record: &'a Record<'a>) -> Option<Self> {
        let target = record.metadata().target();
        let is_defmt = target.starts_with(DEFMT_TARGET_MARKER);
        if !is_defmt {
            return None;
        }

        let timestamp = &target[DEFMT_TARGET_MARKER.len()..];
        Some(Self {
            timestamp,
            log_record: record,
        })
    }

    /// Returns the formatted defmt timestamp.
    pub fn timestamp(&self) -> &str {
        self.timestamp
    }

    pub fn level(&self) -> Level {
        self.log_record.level()
    }

    pub fn args(&self) -> &fmt::Arguments<'a> {
        self.log_record.args()
    }

    pub fn module_path(&self) -> Option<&'a str> {
        self.log_record.module_path()
    }

    pub fn file(&self) -> Option<&'a str> {
        self.log_record.file()
    }

    pub fn line(&self) -> Option<u32> {
        self.log_record.line()
    }

    /// Returns a builder that can format this record for displaying it to the user.
    pub fn printer(&'a self) -> Printer<'a> {
        Printer {
            record: self,
            include_location: false,
            min_timestamp_width: 0,
        }
    }
}

/// Printer for `DefmtRecord`s.
pub struct Printer<'a> {
    record: &'a DefmtRecord<'a>,
    include_location: bool,
    min_timestamp_width: usize,
}

impl<'a> Printer<'a> {
    /// Configure whether to include location info (file, line) in the output.
    ///
    /// If `true`, an additional line will be included in the output that contains file and line
    /// information of the logging statement.
    /// By default, this is `false`.
    pub fn include_location(&mut self, include_location: bool) -> &mut Self {
        self.include_location = include_location;
        self
    }

    /// Pads the defmt timestamp to take up at least the given number of characters.
    pub fn min_timestamp_width(&mut self, min_timestamp_width: usize) -> &mut Self {
        self.min_timestamp_width = min_timestamp_width;
        self
    }

    /// Prints the colored log frame to `sink`.
    ///
    /// The format is as follows (this is not part of the stable API and may change):
    ///
    /// ```text
    /// <timestamp> <level> <args>
    /// └─ <module> @ <file>:<line>
    /// ```
    pub fn print_colored<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        writeln!(
            sink,
            "{timestamp:>0$}{spacing}{level:5} {args}",
            self.min_timestamp_width,
            timestamp = self.record.timestamp(),
            spacing = if self.record.timestamp().is_empty() {
                ""
            } else {
                " "
            },
            level = self
                .record
                .level()
                .to_string()
                .color(color_for_log_level(self.record.level())),
            args = color_diff(self.record.args().to_string()),
        )?;

        if self.include_location {
            let log_record = self.record.log_record;
            print_location(
                sink,
                log_record.file(),
                log_record.line(),
                log_record.module_path(),
            )?;
        }

        Ok(())
    }
}

// color the output of `defmt::assert_eq`
// HACK we should not re-parse formatted output but instead directly format into a color diff
// template; that may require specially tagging log messages that come from `defmt::assert_eq`
fn color_diff(text: String) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let nlines = lines.len();
    if nlines > 2 {
        let left = lines[nlines - 2];
        let right = lines[nlines - 1];

        const LEFT_START: &str = " left: `";
        const RIGHT_START: &str = "right: `";
        const END: &str = "`";
        if left.starts_with(LEFT_START)
            && left.ends_with(END)
            && right.starts_with(RIGHT_START)
            && right.ends_with(END)
        {
            // `defmt::assert_eq!` output
            let left = &left[LEFT_START.len()..left.len() - END.len()];
            let right = &right[RIGHT_START.len()..right.len() - END.len()];

            let mut buf = lines[..nlines - 2].join("\n").bold().to_string();
            buf.push('\n');

            let changeset = Changeset::new(left, right, "");

            writeln!(
                buf,
                "{} {} / {}",
                "diff".bold(),
                "< left".red(),
                "right >".green()
            )
            .ok();
            write!(buf, "{}", "<".red()).ok();
            for diff in &changeset.diffs {
                match diff {
                    Difference::Same(s) => {
                        write!(buf, "{}", s.red()).ok();
                    }
                    Difference::Add(_) => continue,
                    Difference::Rem(s) => {
                        write!(buf, "{}", s.red().bold()).ok();
                    }
                }
            }
            buf.push('\n');

            write!(buf, "{}", ">".green()).ok();
            for diff in &changeset.diffs {
                match diff {
                    Difference::Same(s) => {
                        write!(buf, "{}", s.green()).ok();
                    }
                    Difference::Rem(_) => continue,
                    Difference::Add(s) => {
                        write!(buf, "{}", s.green().bold()).ok();
                    }
                }
            }
            return buf;
        }
    }

    // keep output as it is
    text.bold().to_string()
}

/// Initializes a `log` sink that handles defmt frames.
///
/// Defmt frames will be printed to stdout, other logs to stderr.
///
/// The caller has to provide a `should_log` closure that determines whether a log record should be
/// printed.
///
/// If `always_include_location` is `true`, a second line containing location information will be
/// printed for *all* records, not just for defmt frames (defmt frames always get location info
/// included if it is available, regardless of this setting).
pub fn init_logger(
    always_include_location: bool,
    should_log: impl Fn(&log::Metadata) -> bool + Sync + Send + 'static,
) {
    log::set_boxed_logger(Box::new(Logger {
        always_include_location,
        should_log: Box::new(should_log),
        timing_align: AtomicUsize::new(0),
    }))
    .unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

pub fn init_json_logger(should_log: impl Fn(&log::Metadata) -> bool + Sync + Send + 'static) {
    log::set_boxed_logger(Box::new(JsonLogger::new(Box::new(should_log)))).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

type ShouldLog = Box<dyn Fn(&log::Metadata) -> bool + Sync + Send>;

struct Logger {
    always_include_location: bool,

    should_log: ShouldLog,

    /// Number of characters used by the timestamp. This may increase over time and is used to align
    /// messages.
    timing_align: AtomicUsize,
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match DefmtRecord::new(record) {
            Some(defmt) => {
                // defmt goes to stdout, since it's the primary output produced by this tool.
                let stdout = io::stdout();
                let mut sink = stdout.lock();

                let len = defmt.timestamp().len();
                self.timing_align.fetch_max(len, Ordering::Relaxed);
                let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

                defmt
                    .printer()
                    .include_location(true) // always include location for defmt output
                    .min_timestamp_width(min_timestamp_width)
                    .print_colored(&mut sink)
                    .ok();
            }
            None => {
                // non-defmt logs go to stderr
                let stderr = io::stderr();
                let mut sink = stderr.lock();

                let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

                writeln!(
                    sink,
                    "{timestamp:>0$} {level:5} {args}",
                    min_timestamp_width,
                    timestamp = "(HOST)",
                    level = record
                        .level()
                        .to_string()
                        .color(color_for_log_level(record.level())),
                    args = record.args()
                )
                .ok();

                if self.always_include_location {
                    print_location(
                        &mut sink,
                        record.file(),
                        record.line(),
                        record.module_path(),
                    )
                    .ok();
                }
            }
        }
    }

    fn flush(&self) {}
}

fn color_for_log_level(level: Level) -> Color {
    match level {
        Level::Error => Color::Red,
        Level::Warn => Color::Yellow,
        Level::Info => Color::Green,
        Level::Debug => Color::BrightWhite,
        Level::Trace => Color::BrightBlack,
    }
}

fn print_location<W: io::Write>(
    sink: &mut W,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) -> io::Result<()> {
    if let Some(file) = file {
        // NOTE will always be `Some` if `file` is `Some`
        let mod_path = module_path.unwrap();
        let mut loc = file.to_string();
        if let Some(line) = line {
            loc.push_str(&format!(":{}", line));
        }
        writeln!(sink, "{}", format!("└─ {} @ {}", mod_path, loc).dimmed())?;
    }

    Ok(())
}

struct JsonLogger {
    should_log: ShouldLog,
}

impl Log for JsonLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match DefmtRecord::new(record) {
            Some(record) => {
                let data = record.args();
                let file = record.file();
                let level = record.level().as_str();
                let line = record.line();
                let target_timestamp = record.timestamp();

                let path = record
                    .module_path()
                    .map(|a| a.split("::").collect::<Vec<_>>())
                    .unwrap_or_default();
                // TODO: following 3 lines would panic, if there is no path
                let krate = path[..1][0];
                let function = path[path.len() - 1..][0];
                let modules = &path[1..path.len() - 1];

                let host_timestamp = chrono::Utc::now();

                // defmt goes to stdout, since it's the primary output produced by this tool.
                let stdout = io::stdout();
                let mut sink = stdout.lock();

                writeln!(
                    &mut sink,
                    "{}",
                    // serde_json::to_string(&json!({
                    serde_json::to_string_pretty(&json!({
                        "backtrace": JsonValue::Null,
                        "data": data,
                        "host_timestamp": host_timestamp,
                        "level": level,
                        "location": {
                            "file": file,
                            "line": line,
                            "column": "TODO",
                        },
                        "path": {
                            "crate": krate,
                            "modules": modules,
                            "function": function,
                            "is_method": "TODO",
                        },
                        "target_timestamp": target_timestamp,
                    }))
                    .unwrap(),
                )
                .ok();
            }
            None => {
                // non-defmt logs go to stderr
                let stderr = io::stderr();
                let mut sink = stderr.lock();

                // note: the length of '(HOST)'
                let min_timestamp_width = 6;

                writeln!(
                    sink,
                    "{timestamp:>0$} {level:5} {args}",
                    min_timestamp_width,
                    timestamp = "(HOST)",
                    level = record
                        .level()
                        .to_string()
                        .color(color_for_log_level(record.level())),
                    args = record.args()
                )
                .ok();
            }
        }
    }

    fn flush(&self) {}
}

impl JsonLogger {
    fn new(should_log: ShouldLog) -> Self {
        Self { should_log }
    }
}
