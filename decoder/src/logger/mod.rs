//! This crate provides interoperability utilities between [`defmt`] and the [`log`] crate.
//!
//! If you are implementing a custom defmt decoding tool, this crate can make it easier to integrate
//! it with logs produced with the [`log`] crate.
//!
//! [`log`]: https://crates.io/crates/log
//! [`defmt`]: https://crates.io/crates/defmt

#![cfg(feature = "unstable")]

use crate::decoder::Frame;
use ansi_term::Colour;
use colored::{Color, Colorize};
use difference::{Changeset, Difference};
use log::{Level, Log, Metadata, Record};

use std::{
    fmt::{self, Write as _},
    io::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

const DEFMT_TARGET_MARKER: &str = "defmt@";

/// Logs a defmt frame using the `log` facade.
pub fn log_defmt(
    frame: &Frame<'_>,
    file: Option<&str>,
    line: Option<u32>,
    module_path: Option<&str>,
) {
    let level = match frame.level() {
        crate::decoder::Level::Trace => Level::Trace,
        crate::decoder::Level::Debug => Level::Debug,
        crate::decoder::Level::Info => Level::Info,
        crate::decoder::Level::Warn => Level::Warn,
        crate::decoder::Level::Error => Level::Error,
    };

    let timestamp = frame
        .display_timestamp()
        .map(|display| display.to_string())
        .unwrap_or_default();
    let target = format!("{}{}", DEFMT_TARGET_MARKER, timestamp);
    let display = frame.display_message();

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
}

/// Determines whether `metadata` belongs to a log record produced by [`log_defmt`].
pub fn is_defmt_frame(metadata: &Metadata) -> bool {
    metadata.target().starts_with(DEFMT_TARGET_MARKER)
}

/// A `log` record representing a defmt log frame.
pub struct DefmtRecord<'a> {
    timestamp: &'a str,
    level: Level,
    args: fmt::Arguments<'a>,
    module_path: Option<&'a str>,
    file: Option<&'a str>,
    line: Option<u32>,
}

impl<'a> DefmtRecord<'a> {
    /// If `record` was produced by [`log_defmt`], returns the corresponding `DefmtRecord`.
    pub fn new(record: &Record<'a>) -> Option<Self> {
        let target = record.metadata().target();
        let is_defmt = target.starts_with(DEFMT_TARGET_MARKER);
        if !is_defmt {
            return None;
        }

        let timestamp = &target[DEFMT_TARGET_MARKER.len()..];

        Some(Self {
            level: record.level(),
            timestamp,
            args: *record.args(),
            module_path: record.module_path(),
            file: record.file(),
            line: record.line(),
        })
    }

    /// Returns the formatted defmt timestamp.
    pub fn timestamp(&self) -> &str {
        self.timestamp
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn args(&self) -> &fmt::Arguments<'a> {
        &self.args
    }

    pub fn module_path(&self) -> Option<&'a str> {
        self.module_path
    }

    pub fn file(&self) -> Option<&'a str> {
        self.file
    }

    pub fn line(&self) -> Option<u32> {
        self.line
    }

    /// Returns a builder that can format this record for displaying it to the user.
    pub fn print(&'a self) -> Printer<'a> {
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
    /// Whether to include location info (file, line) in the output.
    ///
    /// If `true`, an additional line will be included in the output that contains file and line
    /// information of the logging statement. By default, this is `false`.
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
        let level_color = match self.record.level() {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::BrightWhite,
            Level::Trace => Color::BrightBlack,
        };

        writeln!(
            sink,
            "{timestamp:>0$} {level:5} {args}",
            self.min_timestamp_width,
            timestamp = self.record.timestamp(),
            level = self.record.level().to_string().color(level_color),
            args = color_diff(self.record.args().to_string()),
        )?;

        if let Some(file) = self.record.file() {
            // NOTE will be `Some` if `file` is `Some`
            let mod_path = self.record.module_path().unwrap();
            // Always include location info for defmt output.
            if self.include_location {
                let mut loc = file.to_string();
                if let Some(line) = self.record.line() {
                    loc.push_str(&format!(":{}", line));
                }
                writeln!(sink, "{}", format!("└─ {} @ {}", mod_path, loc).dimmed())?;
            }
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
            const DARK_RED: Colour = Colour::Fixed(52);
            const DARK_GREEN: Colour = Colour::Fixed(22);

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
                        write!(buf, "{}", Colour::Red.on(DARK_RED).bold().paint(s)).ok();
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
                        write!(buf, "{}", Colour::Green.on(DARK_GREEN).bold().paint(s)).ok();
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
        timing_align: AtomicUsize::new(8),
    }))
    .unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

struct Logger {
    always_include_location: bool,

    should_log: Box<dyn Fn(&log::Metadata) -> bool + Sync + Send>,

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
                    .print()
                    .include_location(true) // always include location for defmt output
                    .min_timestamp_width(min_timestamp_width)
                    .print_colored(&mut sink)
                    .ok();
            }
            None => {
                // non-defmt logs go to stderr
                let stderr = io::stderr();
                let mut sink = stderr.lock();

                let level_color = match record.level() {
                    Level::Error => Color::Red,
                    Level::Warn => Color::Yellow,
                    Level::Info => Color::Green,
                    Level::Debug => Color::BrightWhite,
                    Level::Trace => Color::BrightBlack,
                };

                let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

                writeln!(
                    sink,
                    "{timestamp:>0$} {level:5} {args}",
                    min_timestamp_width,
                    timestamp = "(HOST)",
                    level = record.level().to_string().color(level_color),
                    args = record.args()
                )
                .ok();

                if let Some(file) = record.file() {
                    // NOTE will be `Some` if `file` is `Some`
                    let mod_path = record.module_path().unwrap();
                    if self.always_include_location {
                        let mut loc = file.to_string();
                        if let Some(line) = record.line() {
                            loc.push_str(&format!(":{}", line));
                        }
                        writeln!(sink, "{}", format!("└─ {} @ {}", mod_path, loc).dimmed()).ok();
                    }
                }
            }
        }
    }

    fn flush(&self) {}
}
