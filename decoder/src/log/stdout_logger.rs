use colored::{Color, ColoredString, Colorize, Styles};
use dissimilar::Chunk;
use log::{Level, Log, Metadata, Record as LogRecord};

use std::{
    fmt::Write as _,
    io::{self, StderrLock, StdoutLock},
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{
    format::{self, Alignment, LogColor, LogFormat, LogMetadata, LogSegment},
    DefmtLoggerInfo, DefmtRecord,
};

enum Record<'a> {
    Defmt(&'a DefmtRecord<'a>),
    Host(&'a LogRecord<'a>),
}

pub(crate) struct StdoutLogger {
    log_format: Vec<LogSegment>,
    host_log_format: Vec<LogSegment>,
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
                if record.level().is_some() {
                    self.print_defmt_record(record, sink);
                } else {
                    self.print_defmt_record_without_format(record, sink);
                }
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
        host_log_format: Option<&str>,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Box<Self> {
        Box::new(Self::new_unboxed(log_format, host_log_format, should_log))
    }

    pub fn new_unboxed(
        log_format: Option<&str>,
        host_log_format: Option<&str>,
        should_log: impl Fn(&Metadata) -> bool + Sync + Send + 'static,
    ) -> Self {
        const DEFAULT_LOG_FORMAT: &str = "{t} {L} {s}\n└─ {m} @ {F}:{l}";
        const DEFAULT_HOST_LOG_FORMAT: &str = "(HOST) {L} {s}";

        let log_format = log_format.unwrap_or(DEFAULT_LOG_FORMAT);
        let log_format = format::parse(log_format).unwrap();

        let host_log_format = host_log_format.unwrap_or(DEFAULT_HOST_LOG_FORMAT);
        let host_log_format = format::parse(host_log_format).unwrap();

        Self {
            log_format,
            host_log_format,
            should_log: Box::new(should_log),
            timing_align: AtomicUsize::new(0),
        }
    }

    pub fn info(&self) -> DefmtLoggerInfo {
        let has_timestamp = self
            .log_format
            .iter()
            .any(|s| s.metadata == LogMetadata::Timestamp);
        DefmtLoggerInfo { has_timestamp }
    }

    fn print_defmt_record(&self, record: DefmtRecord, mut sink: StdoutLock) {
        let len = record.timestamp().len();
        self.timing_align.fetch_max(len, Ordering::Relaxed);
        let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);

        Printer::new(Record::Defmt(&record), &self.log_format)
            .min_timestamp_width(min_timestamp_width)
            .print_frame(&mut sink)
            .ok();
    }

    pub(super) fn print_defmt_record_without_format(
        &self,
        record: DefmtRecord,
        mut sink: StdoutLock,
    ) {
        const RAW_FORMAT: &[LogSegment] = &[LogSegment::new(LogMetadata::Log)];
        Printer::new(Record::Defmt(&record), RAW_FORMAT)
            .print_frame(&mut sink)
            .ok();
    }

    pub(super) fn print_host_record(&self, record: &LogRecord, mut sink: StderrLock) {
        let min_timestamp_width = self.timing_align.load(Ordering::Relaxed);
        Printer::new(Record::Host(record), &self.host_log_format)
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
            match &segment.metadata {
                LogMetadata::String(s) => self.print_string(sink, s),
                LogMetadata::Timestamp => self.print_timestamp(sink, &segment.format),
                LogMetadata::FileName => self.print_file_name(sink, &segment.format),
                LogMetadata::FilePath => self.print_file_path(sink, &segment.format),
                LogMetadata::ModulePath => self.print_module_path(sink, &segment.format),
                LogMetadata::LineNumber => self.print_line_number(sink, &segment.format),
                LogMetadata::LogLevel => self.print_log_level(sink, &segment.format),
                LogMetadata::Log => self.print_log(sink, &segment.format),
            }?;
        }
        writeln!(sink)
    }

    fn print_string<W: io::Write>(&self, sink: &mut W, s: &str) -> io::Result<()> {
        write!(sink, "{s}")
    }

    fn print_timestamp<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        let timestamp = match self.record {
            Record::Defmt(record) => {
                if record.timestamp().is_empty() {
                    String::from("<time>")
                } else {
                    record.timestamp().to_string()
                }
            }
            Record::Host(_) => String::from("<time>"),
        };

        let timestamp = apply_format(
            timestamp.as_str(),
            format.color,
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(self.min_timestamp_width);

        write_string(timestamp, sink, width, format.alignment)
    }

    fn print_log_level<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        let level = if let Some(level) = self.record_log_level() {
            level.to_string()
        } else {
            String::from("<lvl>")
        };

        let color = format.color.unwrap_or(LogColor::SeverityLevel);
        let level = apply_format(
            level.as_str(),
            Some(color),
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(5);

        write_string(level, sink, width, format.alignment)
    }

    fn print_file_path<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        let file_path = match self.record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        }
        .unwrap_or("<file>");

        let file_path = apply_format(
            file_path,
            format.color,
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(0);

        write_string(file_path, sink, width, format.alignment)
    }

    fn print_file_name<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
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

        let file_name = apply_format(
            file_name,
            format.color,
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(0);

        write_string(file_name, sink, width, format.alignment)
    }

    fn print_module_path<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        let module_path = match self.record {
            Record::Defmt(record) => record.module_path(),
            Record::Host(record) => record.module_path(),
        }
        .unwrap_or("<mod path>");

        let module_path = apply_format(
            module_path,
            format.color,
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(0);

        write_string(module_path, sink, width, format.alignment)
    }

    fn print_line_number<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        let line_number = match self.record {
            Record::Defmt(record) => record.line(),
            Record::Host(record) => record.line(),
        }
        .unwrap_or(0)
        .to_string();

        let line_number = apply_format(
            line_number.as_str(),
            format.color,
            format.style,
            self.record_log_level(),
        );
        let width = format.width.unwrap_or(4);

        write_string(line_number, sink, width, format.alignment)
    }

    fn print_log<W: io::Write>(&self, sink: &mut W, format: &LogFormat) -> io::Result<()> {
        match self.record {
            Record::Defmt(record) => match color_diff(record.args().to_string()) {
                Ok(s) => write!(sink, "{s}"),
                Err(s) => {
                    let log = apply_format(
                        s.as_str(),
                        format.color,
                        format.style,
                        self.record_log_level(),
                    );
                    let width = format.width.unwrap_or(0);
                    write_string(log, sink, width, format.alignment)
                }
            },
            Record::Host(record) => {
                let s = record.args().to_string();
                write!(sink, "{s}")
            }
        }
    }

    fn record_log_level(&self) -> Option<Level> {
        match self.record {
            Record::Defmt(record) => record.level(),
            Record::Host(record) => Some(record.level()),
        }
    }
}

// color the output of `defmt::assert_eq`
// HACK we should not re-parse formatted output but instead directly format into a color diff
// template; that may require specially tagging log messages that come from `defmt::assert_eq`
fn color_diff(text: String) -> Result<String, String> {
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
            return Ok(buf);
        }
    }

    Err(text)
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

fn apply_format(
    s: &str,
    color: Option<LogColor>,
    style: Option<Styles>,
    level: Option<Level>,
) -> ColoredString {
    let s = ColoredString::from(s);
    let s = apply_color(s, color, level);
    let s = apply_style(s, style);
    s
}

fn apply_color(
    s: ColoredString,
    log_color: Option<LogColor>,
    level: Option<Level>,
) -> ColoredString {
    match log_color {
        Some(color) => match color {
            LogColor::Color(c) => s.color(c),
            LogColor::SeverityLevel => match level {
                Some(level) => s.color(color_for_log_level(level)),
                None => s,
            },
            LogColor::WarnError => match level {
                Some(level @ (Level::Warn | Level::Error)) => s.color(color_for_log_level(level)),
                _ => s,
            },
        },
        None => s,
    }
}

fn apply_style(s: ColoredString, log_style: Option<Styles>) -> ColoredString {
    match log_style {
        Some(log_style) => match log_style {
            Styles::Bold => s.bold(),
            Styles::Italic => s.italic(),
            Styles::Underline => s.underline(),
            Styles::Strikethrough => s.strikethrough(),
            Styles::Dimmed => s.dimmed(),
            Styles::Clear => s.clear(),
            Styles::Reversed => s.reversed(),
            Styles::Blink => s.blink(),
            Styles::Hidden => s.hidden(),
        },
        None => s,
    }
}

fn write_string<W: io::Write>(
    s: ColoredString,
    sink: &mut W,
    width: usize,
    alignment: Option<Alignment>,
) -> io::Result<()> {
    match alignment.unwrap_or(Alignment::Left) {
        Alignment::Left => write!(sink, "{s:<0$}", width),
        Alignment::Center => write!(sink, "{s:^0$}", width),
        Alignment::Right => write!(sink, "{s:>0$}", width),
    }
}
