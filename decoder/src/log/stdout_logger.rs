use colored::{Color, Colorize};
use dissimilar::Chunk;
use log::{Level, Log, Metadata, Record as LogRecord};

use std::{
    fmt::Write as _,
    io::{self, StderrLock, StdoutLock},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::format;
use super::{format::LogSegment, DefmtRecord};

enum Record<'a> {
    Defmt(&'a DefmtRecord<'a>),
    Host(&'a LogRecord<'a>),
}

pub(crate) struct StdoutLogger {
    format: Vec<LogSegment>,
    should_log: Box<dyn Fn(&Metadata) -> bool + Sync + Send>,
    /// Number of characters used by the timestamp.
    /// This may increase over time and is used to align messages.
    timing_align: AtomicUsize,
}

impl Log for StdoutLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        (self.should_log)(metadata)
    }

    fn log(&self, record: &LogRecord) {
        if !self.enabled(record.metadata()) {
            return;
        }

        match DefmtRecord::new(record) {
            Some(record) => {
                // defmt goes to stdout, since it's the primary output produced by this tool.
                let sink = io::stdout().lock();
                self.print_defmt_record(record, sink);
            }
            None => {
                // non-defmt logs go to stderr
                let sink = io::stderr().lock();
                self.print_host_record(record, sink);
            }
        }
    }

    fn flush(&self) {}
}

impl StdoutLogger {
    pub fn new(
        log_format: Option<&str>,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Box<Self> {
        Box::new(Self::new_unboxed(log_format, should_log))
    }

    pub fn new_unboxed(
        log_format: Option<&str>,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Self {
        const DEFAULT_LOG_FORMAT: &str = "{t} {L} {s}\n└─ {m} @ {F}:{l}";

        let format = log_format.unwrap_or(DEFAULT_LOG_FORMAT);
        let format = format::parse(format).unwrap_or_else(|_| {
            // Use the default format if the user-provided format is invalid
            format::parse(DEFAULT_LOG_FORMAT).unwrap()
        });

        Self {
            format,
            should_log: Box::new(should_log),
            timing_align: AtomicUsize::new(0),
        }
    }

    fn print_defmt_record(&self, record: DefmtRecord, mut sink: StdoutLock) {
        let len = record.timestamp().len();
        self.timing_align.fetch_max(len, Ordering::Relaxed);
        let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

        Printer::new(Record::Defmt(&record), &self.format)
            .min_timestamp_width(min_timestamp_width)
            .print_frame(&mut sink)
            .ok();
    }

    pub(super) fn print_host_record(&self, record: &LogRecord, mut sink: StderrLock) {
        let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);
        Printer::new(Record::Host(record), &self.format)
            .min_timestamp_width(min_timestamp_width)
            .print_frame(&mut sink)
            .ok();
    }
}

/// Printer for `DefmtRecord`s.
struct Printer<'a> {
    record: Record<'a>,
    format: &'a [LogSegment],
    min_timestamp_width: usize,
}

impl<'a> Printer<'a> {
    pub fn new(record: Record<'a>, format: &'a [LogSegment]) -> Self {
        Self {
            record,
            format,
            min_timestamp_width: 0,
        }
    }

    /// Pads the defmt timestamp to take up at least the given number of characters.
    pub fn min_timestamp_width(&mut self, min_timestamp_width: usize) -> &mut Self {
        self.min_timestamp_width = min_timestamp_width;
        self
    }

    /// Prints the formatted log frame to `sink`.
    pub fn print_frame<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        for segment in self.format {
            match segment {
                LogSegment::String(s) => self.print_string(sink, s),
                LogSegment::Timestamp => self.print_timestamp(sink),
                LogSegment::FileName => self.print_file_name(sink),
                LogSegment::FilePath => self.print_file_path(sink),
                LogSegment::ModulePath => self.print_module_path(sink),
                LogSegment::LineNumber => self.print_line_number(sink),
                LogSegment::LogLevel => self.print_log_level(sink),
                LogSegment::Log => self.print_log(sink),
            }?;
        }
        writeln!(sink)
    }

    fn print_string<W: io::Write>(&self, sink: &mut W, s: &str) -> io::Result<()> {
        write!(sink, "{s}")
    }

    fn print_timestamp<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let timestamp = match self.record {
            Record::Defmt(record) => record.timestamp().to_string(),
            Record::Host(_) => String::from("(HOST)"),
        };

        write!(sink, "{timestamp:>0$}", self.min_timestamp_width,)
    }

    fn print_log_level<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let level = match self.record {
            Record::Defmt(record) => record.level(),
            Record::Host(record) => Some(record.level()),
        };

        let level = if let Some(level) = level {
            // TODO: Should the color be customizable via the format too?
            level
                .to_string()
                .color(color_for_log_level(level))
                .to_string()
        } else {
            String::from("<lvl>")
        };

        write!(sink, "{level:5}")
    }

    fn print_file_path<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let file_path = match self.record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        }
        .unwrap_or("<file>");

        write!(sink, "{file_path}")
    }

    fn print_file_name<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let file = match self.record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        };

        let file_name = if let Some(file) = file {
            let file_name = Path::new(file).file_name();
            if let Some(file_name) = file_name {
                file_name.to_str().unwrap_or("<file>")
            } else {
                "<file>"
            }
        } else {
            "<file>"
        };

        write!(sink, "{file_name}")
    }

    fn print_module_path<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let module_path = match self.record {
            Record::Defmt(record) => record.module_path(),
            Record::Host(record) => record.module_path(),
        }
        .unwrap_or("<mod path>");

        write!(sink, "{module_path}")
    }

    fn print_line_number<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let line_number = match self.record {
            Record::Defmt(record) => record.line(),
            Record::Host(record) => record.line(),
        }
        .unwrap_or(0);

        write!(sink, "{line_number}")
    }

    fn print_log<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        let args = match self.record {
            Record::Defmt(record) => record.args(),
            Record::Host(record) => record.args(),
        };

        write!(sink, "{log}", log = color_diff(args.to_string()),)
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

            let diffs = dissimilar::diff(left, right);

            writeln!(
                buf,
                "{} {} / {}",
                "diff".bold(),
                "< left".red(),
                "right >".green()
            )
            .ok();
            write!(buf, "{}", "<".red()).ok();
            for diff in &diffs {
                match diff {
                    Chunk::Equal(s) => {
                        write!(buf, "{}", s.red()).ok();
                    }
                    Chunk::Insert(_) => continue,
                    Chunk::Delete(s) => {
                        write!(buf, "{}", s.red().bold()).ok();
                    }
                }
            }
            buf.push('\n');

            write!(buf, "{}", ">".green()).ok();
            for diff in &diffs {
                match diff {
                    Chunk::Equal(s) => {
                        write!(buf, "{}", s.green()).ok();
                    }
                    Chunk::Delete(_) => continue,
                    Chunk::Insert(s) => {
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
