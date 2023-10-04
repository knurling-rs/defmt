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
        let log_format = format::parse(log_format)
            .unwrap_or_else(|_| panic!("log format is invalid '{log_format}'"));

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
pub(super) struct Printer<'a> {
    record: Record<'a>,
    format: &'a [LogSegment],
    min_timestamp_width: usize,
}

impl<'a> Printer<'a> {
    fn new(record: Record<'a>, format: &'a [LogSegment]) -> Self {
        Self {
            record,
            format,
            min_timestamp_width: 0,
        }
    }

    pub fn new_defmt(record: &'a DefmtRecord<'a>, format: &'a [LogSegment]) -> Self {
        Self::new(Record::Defmt(record), format)
    }

    /// Pads the defmt timestamp to take up at least the given number of characters.
    /// TODO: Remove this, shouldn't be needed now that we have width field support
    fn min_timestamp_width(&mut self, min_timestamp_width: usize) -> &mut Self {
        self.min_timestamp_width = min_timestamp_width;
        self
    }

    /// Prints the formatted log frame to `sink`.
    pub fn print_frame<W: io::Write>(&self, sink: &mut W) -> io::Result<()> {
        for segment in self.format {
            let s = self.build_segment(segment);
            write!(sink, "{s}")?;
        }
        writeln!(sink)
    }

    pub(super) fn format_frame(&self) -> String {
        let mut buf = String::new();
        for segment in self.format {
            let s = self.build_segment(segment);
            write!(buf, "{s}").expect("writing to String cannot fail");
        }
        buf
    }

    fn build_segment(&self, segment: &LogSegment) -> String {
        match &segment.metadata {
            LogMetadata::String(s) => s.to_string(),
            LogMetadata::Timestamp => self.build_timestamp(&segment.format),
            LogMetadata::FileName => self.build_file_name(&segment.format),
            LogMetadata::FilePath => self.build_file_path(&segment.format),
            LogMetadata::ModulePath => self.build_module_path(&segment.format),
            LogMetadata::LineNumber => self.build_line_number(&segment.format),
            LogMetadata::LogLevel => self.build_log_level(&segment.format),
            LogMetadata::Log => self.build_log(&segment.format),
            LogMetadata::NestedLogSegments(segments) => {
                self.build_nested(segments, &segment.format)
            }
        }
    }

    fn build_nested(&self, segments: &[LogSegment], format: &LogFormat) -> String {
        let mut result = String::new();
        for segment in segments {
            let s = match &segment.metadata {
                LogMetadata::String(s) => s.to_string(),
                LogMetadata::Timestamp => self.build_timestamp(&segment.format),
                LogMetadata::FileName => self.build_file_name(&segment.format),
                LogMetadata::FilePath => self.build_file_path(&segment.format),
                LogMetadata::ModulePath => self.build_module_path(&segment.format),
                LogMetadata::LineNumber => self.build_line_number(&segment.format),
                LogMetadata::LogLevel => self.build_log_level(&segment.format),
                LogMetadata::Log => self.build_log(&segment.format),
                LogMetadata::NestedLogSegments(segments) => {
                    self.build_nested(segments, &segment.format)
                }
            };
            result.push_str(&s);
        }

        build_formatted_string(&result, format, 0, self.record_log_level(), format.color)
    }

    fn build_timestamp(&self, format: &LogFormat) -> String {
        let s = match self.record {
            Record::Defmt(record) if !record.timestamp().is_empty() => record.timestamp(),
            _ => "<time>",
        }
        .to_string();

        build_formatted_string(
            s.as_str(),
            format,
            self.min_timestamp_width,
            self.record_log_level(),
            format.color,
        )
    }

    fn build_log_level(&self, format: &LogFormat) -> String {
        let s = match self.record_log_level() {
            Some(level) => level.to_string(),
            None => "<lvl>".to_string(),
        };

        let color = format.color.unwrap_or(LogColor::SeverityLevel);

        build_formatted_string(s.as_str(), format, 5, self.record_log_level(), Some(color))
    }

    fn build_file_path(&self, format: &LogFormat) -> String {
        let file_path = match self.record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        }
        .unwrap_or("<file>");

        build_formatted_string(file_path, format, 0, self.record_log_level(), format.color)
    }

    fn build_file_name(&self, format: &LogFormat) -> String {
        let file = match self.record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        };

        let s = if let Some(file) = file {
            let file_name = Path::new(file).file_name();
            if let Some(file_name) = file_name {
                file_name.to_str().unwrap_or("<file>")
            } else {
                "<file>"
            }
        } else {
            "<file>"
        };

        build_formatted_string(s, format, 0, self.record_log_level(), format.color)
    }

    fn build_module_path(&self, format: &LogFormat) -> String {
        let s = match self.record {
            Record::Defmt(record) => record.module_path(),
            Record::Host(record) => record.module_path(),
        }
        .unwrap_or("<mod path>");

        build_formatted_string(s, format, 0, self.record_log_level(), format.color)
    }

    fn build_line_number(&self, format: &LogFormat) -> String {
        let s = match self.record {
            Record::Defmt(record) => record.line(),
            Record::Host(record) => record.line(),
        }
        .unwrap_or(0)
        .to_string();

        build_formatted_string(s.as_str(), format, 4, self.record_log_level(), format.color)
    }

    fn build_log(&self, format: &LogFormat) -> String {
        match self.record {
            Record::Defmt(record) => match color_diff(record.args().to_string()) {
                Ok(s) => s.to_string(),
                Err(s) => build_formatted_string(
                    s.as_str(),
                    format,
                    0,
                    self.record_log_level(),
                    format.color,
                ),
            },
            Record::Host(record) => record.args().to_string(),
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

fn apply_styles(s: ColoredString, log_style: Option<&Vec<Styles>>) -> ColoredString {
    let Some(log_styles) = log_style else {
        return s;
    };

    let mut stylized_string = s;
    for style in log_styles {
        stylized_string = match style {
            Styles::Bold => stylized_string.bold(),
            Styles::Italic => stylized_string.italic(),
            Styles::Underline => stylized_string.underline(),
            Styles::Strikethrough => stylized_string.strikethrough(),
            Styles::Dimmed => stylized_string.dimmed(),
            Styles::Clear => stylized_string.clear(),
            Styles::Reversed => stylized_string.reversed(),
            Styles::Blink => stylized_string.blink(),
            Styles::Hidden => stylized_string.hidden(),
        };
    }

    stylized_string
}

fn build_formatted_string(
    s: &str,
    format: &LogFormat,
    default_width: usize,
    level: Option<Level>,
    log_color: Option<LogColor>,
) -> String {
    let s = ColoredString::from(s);
    let s = apply_color(s, log_color, level);
    let colored_str = apply_styles(s, format.style.as_ref());

    let alignment = format.alignment.unwrap_or(Alignment::Left);
    let width = format.width.unwrap_or(default_width);

    let mut result = String::new();
    match alignment {
        Alignment::Left => write!(&mut result, "{colored_str:<0$}", width),
        Alignment::Center => write!(&mut result, "{colored_str:^0$}", width),
        Alignment::Right => write!(&mut result, "{colored_str:>0$}", width),
    }
    .expect("Failed to format string: \"{colored_str}\"");
    result
}
