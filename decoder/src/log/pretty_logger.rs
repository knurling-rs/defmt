use colored::{Color, Colorize};
use difference::{Changeset, Difference};
use log::{Level, Log, Metadata, Record};

use std::{
    fmt::Write as _,
    io::{self, StderrLock, StdoutLock, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

use super::DefmtRecord;

pub(crate) struct PrettyLogger {
    always_include_location: bool,
    should_log: Box<dyn Fn(&Metadata) -> bool + Sync + Send>,
    /// Number of characters used by the timestamp. This may increase over time and is used to align
    /// messages.
    timing_align: AtomicUsize,
}

impl Log for PrettyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match DefmtRecord::new(record) {
            Some(record) => {
                // defmt goes to stdout, since it's the primary output produced by this tool.
                let stdout = io::stdout();
                let sink = stdout.lock();

                match record.level() {
                    Some(level) => self.print_defmt_record(record, level, sink),
                    None => Self::print_println_record(record, sink),
                };
            }
            None => {
                // non-defmt logs go to stderr
                let stderr = io::stderr();
                let sink = stderr.lock();

                self.print_host_record(record, sink);
            }
        }
    }

    fn flush(&self) {}
}

impl PrettyLogger {
    pub fn new(
        always_include_location: bool,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Box<Self> {
        Box::new(Self::new_unboxed(always_include_location, should_log))
    }

    pub fn new_unboxed(
        always_include_location: bool,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Self {
        Self {
            always_include_location,
            should_log: Box::new(should_log),
            timing_align: AtomicUsize::new(0),
        }
    }

    fn print_defmt_record(&self, record: DefmtRecord, level: Level, mut sink: StdoutLock) {
        let len = record.timestamp().len();
        self.timing_align.fetch_max(len, Ordering::Relaxed);
        let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

        Printer::new(&record, level)
            .include_location(true) // always include location for defmt output
            .min_timestamp_width(min_timestamp_width)
            .print_colored(&mut sink)
            .ok();
    }

    pub(super) fn print_host_record(&self, record: &Record, mut sink: StderrLock) {
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

    fn print_println_record(record: DefmtRecord, mut sink: StdoutLock) {
        writeln!(&mut sink, "{}{}", record.timestamp(), record.args()).ok();
        print_location(
            &mut sink,
            record.file(),
            record.line(),
            record.module_path(),
        )
        .ok();
    }
}

/// Printer for `DefmtRecord`s.
pub struct Printer<'a> {
    record: &'a DefmtRecord<'a>,
    include_location: bool,
    level: Level,
    min_timestamp_width: usize,
}

impl<'a> Printer<'a> {
    pub fn new(record: &'a DefmtRecord, level: Level) -> Self {
        Self {
            record,
            include_location: false,
            level,
            min_timestamp_width: 0,
        }
    }

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
                .level
                .to_string()
                .color(color_for_log_level(self.level)),
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
