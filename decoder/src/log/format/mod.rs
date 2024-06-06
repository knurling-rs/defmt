use super::{DefmtRecord, Payload};
use crate::Frame;
use colored::{Color, ColoredString, Colorize, Styles};
use dissimilar::Chunk;
use log::{Level, Record as LogRecord};
use regex::Regex;
use std::{fmt::Write, path::Path};

mod parser;

/// Representation of what a [LogSegment] can be.
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub(super) enum LogMetadata {
    /// `{c}` format specifier.
    ///
    /// Prints the name of the crate where the log is coming from.
    CrateName,

    /// `{f}` format specifier.
    ///
    /// This specifier may be repeated up to 255 times.
    /// For a file "/path/to/crate/src/foo/bar.rs":
    /// - `{f}` prints "bar.rs".
    /// - `{ff}` prints "foo/bar.rs".
    /// - `{fff}` prints "src/foo/bar.rs"
    FileName(u8),

    /// `{F}` format specifier.
    ///
    /// For a file "/path/to/crate/src/foo/bar.rs"
    /// this option prints "/path/to/crate/src/foo/bar.rs".
    FilePath,

    /// `{l}` format specifier.
    ///
    /// Prints the line number where the log is coming from.
    LineNumber,

    /// `{s}` format specifier.
    ///
    /// Prints the actual log contents.
    /// For `defmt::info!("hello")`, this prints "hello".
    Log,

    /// `{L}` format specifier.
    ///
    /// Prints the log level.
    /// For `defmt::info!("hello")`, this prints "INFO".
    LogLevel,

    /// `{m}` format specifier.
    ///
    /// Prints the module path of the function where the log is coming from.
    /// For the following log:
    ///
    /// ```ignore
    /// // crate: my_crate
    /// mod foo {
    ///     fn bar() {
    ///         defmt::info!("hello");
    ///     }
    /// }
    /// ```
    /// this prints "my_crate::foo::bar".
    ModulePath,

    /// Represents the parts of the formatting string that is not specifiers.
    String(String),

    /// `{t}` format specifier.
    ///
    /// Prints the timestamp at which something was logged.
    /// For a log printed with a timestamp 123456 ms, this prints "123456".
    Timestamp,

    /// Represents formats specified within nested curly brackets in the formatting string.
    NestedLogSegments(Vec<LogSegment>),
}

impl LogMetadata {
    /// Checks whether this `LogMetadata` came from a specifier such as
    /// {t}, {f}, etc.
    fn is_metadata_specifier(&self) -> bool {
        !matches!(
            self,
            LogMetadata::String(_) | LogMetadata::NestedLogSegments(_)
        )
    }
}

/// Coloring options for [LogSegment]s.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum LogColor {
    /// User-defined color.
    ///
    /// Use a string that can be parsed by the FromStr implementation
    /// of [colored::Color].
    Color(colored::Color),

    /// Color matching the default color for the log level.
    /// Use `"severity"` as a format parameter to use this option.
    SeverityLevel,

    /// Color matching the default color for the log level,
    /// but only if the log level is WARN or ERROR.
    ///
    /// Use `"werror"` as a format parameter to use this option.
    WarnError,
}

/// Alignment options for [LogSegment]s.
#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Alignment {
    Center,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Padding {
    Space,
    Zero,
}

/// Representation of a segment of the formatting string.
#[derive(Debug, PartialEq, Clone)]
pub(super) struct LogSegment {
    pub(super) metadata: LogMetadata,
    pub(super) format: LogFormat,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub(super) struct LogFormat {
    pub(super) width: Option<usize>,
    pub(super) color: Option<LogColor>,
    pub(super) style: Option<Vec<colored::Styles>>,
    pub(super) alignment: Option<Alignment>,
    pub(super) padding: Option<Padding>,
}

impl LogSegment {
    pub(super) const fn new(metadata: LogMetadata) -> Self {
        Self {
            metadata,
            format: LogFormat {
                color: None,
                style: None,
                width: None,
                alignment: None,
                padding: None,
            },
        }
    }

    #[cfg(test)]
    pub(crate) const fn with_color(mut self, color: LogColor) -> Self {
        self.format.color = Some(color);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_style(mut self, style: colored::Styles) -> Self {
        let mut styles = self.format.style.unwrap_or_default();
        styles.push(style);
        self.format.style = Some(styles);
        self
    }

    #[cfg(test)]
    pub(crate) const fn with_width(mut self, width: usize) -> Self {
        self.format.width = Some(width);
        self
    }

    #[cfg(test)]
    pub(crate) const fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.format.alignment = Some(alignment);
        self
    }

    #[cfg(test)]
    pub(crate) const fn with_padding(mut self, padding: Padding) -> Self {
        self.format.padding = Some(padding);
        self
    }
}

pub struct Formatter {
    formatter: InternalFormatter,
}

impl Formatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            formatter: InternalFormatter::new(config, Source::Defmt),
        }
    }

    pub fn format_frame<'a>(
        &self,
        frame: Frame<'a>,
        file: Option<&'a str>,
        line: Option<u32>,
        module_path: Option<&str>,
    ) -> String {
        let (timestamp, level) = super::timestamp_and_level_from_frame(&frame);

        // HACK: use match instead of let, because otherwise compilation fails
        #[allow(clippy::match_single_binding)]
        match format_args!("{}", frame.display_message()) {
            args => {
                let log_record = &LogRecord::builder()
                    .args(args)
                    .module_path(module_path)
                    .file(file)
                    .line(line)
                    .build();

                let record = DefmtRecord {
                    log_record,
                    payload: Payload { level, timestamp },
                };

                self.format(&record)
            }
        }
    }

    pub(super) fn format(&self, record: &DefmtRecord) -> String {
        self.formatter.format(&Record::Defmt(record))
    }
}

pub struct HostFormatter {
    formatter: InternalFormatter,
}

impl HostFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            formatter: InternalFormatter::new(config, Source::Host),
        }
    }

    pub fn format(&self, record: &LogRecord) -> String {
        self.formatter.format(&Record::Host(record))
    }
}

#[derive(Debug)]
struct InternalFormatter {
    format: Vec<LogSegment>,
}

#[derive(Clone, Copy, PartialEq)]
enum Source {
    Defmt,
    Host,
}

enum Record<'a> {
    Defmt(&'a DefmtRecord<'a>),
    Host(&'a LogRecord<'a>),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum FormatterFormat<'a> {
    Default { with_location: bool },
    Legacy { with_location: bool },
    Custom(&'a str),
}

impl Default for FormatterFormat<'_> {
    fn default() -> Self {
        FormatterFormat::Default {
            with_location: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct FormatterConfig<'a> {
    pub format: FormatterFormat<'a>,
    pub is_timestamp_available: bool,
}

impl<'a> FormatterConfig<'a> {
    pub fn custom(format: &'a str) -> Self {
        FormatterConfig {
            format: FormatterFormat::Custom(format),
            is_timestamp_available: false,
        }
    }

    pub fn with_timestamp(mut self) -> Self {
        self.is_timestamp_available = true;
        self
    }

    pub fn with_location(mut self) -> Self {
        // TODO: Should we warn the user that trying to set a location
        //       for a custom format won't work?
        match self.format {
            FormatterFormat::Default { with_location: _ } => {
                self.format = FormatterFormat::Default {
                    with_location: true,
                };
                self
            }
            _ => self,
        }
    }
}

impl InternalFormatter {
    fn new(config: FormatterConfig, source: Source) -> Self {
        const FORMAT: &str = "{{[{L}]%bold} {s}%werror}";
        const FORMAT_WITH_LOCATION: &str = "{{[{L}]%bold} {{c:bold}/{ff}:{l}%45} {s}%werror}";
        const FORMAT_WITH_TIMESTAMP: &str = "{{t:>8} {[{L}]%bold} {s}%werror}";
        const FORMAT_WITH_TIMESTAMP_AND_LOCATION: &str =
            "{{t:>8} {[{L}]%bold} {{c:bold}/{ff}:{l}%45} {s}%werror}";

        const LEGACY_FORMAT: &str = "{L} {s}";
        const LEGACY_FORMAT_WITH_LOCATION: &str = "{L} {s}\n└─ {m} @ {F}:{l}";
        const LEGACY_FORMAT_WITH_TIMESTAMP: &str = "{t} {L} {s}";
        const LEGACY_FORMAT_WITH_TIMESTAMP_AND_LOCATION: &str = "{t} {L} {s}\n└─ {m} @ {F}:{l}";

        let format = match config.format {
            FormatterFormat::Default { with_location } => {
                let mut format = match (with_location, config.is_timestamp_available) {
                    (false, false) => FORMAT,
                    (false, true) => FORMAT_WITH_TIMESTAMP,
                    (true, false) => FORMAT_WITH_LOCATION,
                    (true, true) => FORMAT_WITH_TIMESTAMP_AND_LOCATION,
                }
                .to_string();

                if source == Source::Host {
                    format.insert_str(0, "(HOST) ");
                }

                format
            }
            FormatterFormat::Legacy { with_location } => {
                let mut format = match (with_location, config.is_timestamp_available) {
                    (false, false) => LEGACY_FORMAT,
                    (false, true) => LEGACY_FORMAT_WITH_TIMESTAMP,
                    (true, false) => LEGACY_FORMAT_WITH_LOCATION,
                    (true, true) => LEGACY_FORMAT_WITH_TIMESTAMP_AND_LOCATION,
                }
                .to_string();

                if source == Source::Host {
                    format.insert_str(0, "(HOST) ");
                }

                format
            }
            FormatterFormat::Custom(format) => format.to_string(),
        };

        let format = parser::parse(&format).expect("log format is invalid '{format}'");

        if matches!(config.format, FormatterFormat::Custom(_)) {
            let format_has_timestamp = format_has_timestamp(&format);
            if format_has_timestamp && !config.is_timestamp_available {
                log::warn!(
                    "logger format contains timestamp but no timestamp implementation \
                    was provided; consider removing the timestamp (`{{t}}`) from the \
                    logger format or provide a `defmt::timestamp!` implementation"
                );
            } else if !format_has_timestamp && config.is_timestamp_available {
                log::warn!(
                    "`defmt::timestamp!` implementation was found, but timestamp is not \
                    part of the log format; consider adding the timestamp (`{{t}}`) \
                    argument to the log format"
                );
            }
        }

        Self { format }
    }

    fn format(&self, record: &Record) -> String {
        let mut buf = String::new();
        // Only format logs, not printlns
        // printlns do not have a log level
        if get_log_level_of_record(record).is_some() {
            for segment in &self.format {
                let s = self.build_segment(record, segment);
                write!(buf, "{s}").expect("writing to String cannot fail");
            }
        } else {
            let empty_format: LogFormat = Default::default();
            let s = self.build_log(record, &empty_format);
            write!(buf, "{s}").expect("writing to String cannot fail");
        }
        buf
    }

    fn build_segment(&self, record: &Record, segment: &LogSegment) -> String {
        match &segment.metadata {
            LogMetadata::String(s) => s.to_string(),
            LogMetadata::Timestamp => self.build_timestamp(record, &segment.format),
            LogMetadata::CrateName => self.build_crate_name(record, &segment.format),
            LogMetadata::FileName(n) => self.build_file_name(record, &segment.format, *n),
            LogMetadata::FilePath => self.build_file_path(record, &segment.format),
            LogMetadata::ModulePath => self.build_module_path(record, &segment.format),
            LogMetadata::LineNumber => self.build_line_number(record, &segment.format),
            LogMetadata::LogLevel => self.build_log_level(record, &segment.format),
            LogMetadata::Log => self.build_log(record, &segment.format),
            LogMetadata::NestedLogSegments(segments) => {
                self.build_nested(record, segments, &segment.format)
            }
        }
    }

    fn build_nested(&self, record: &Record, segments: &[LogSegment], format: &LogFormat) -> String {
        let mut result = String::new();
        for segment in segments {
            let s = match &segment.metadata {
                LogMetadata::String(s) => s.to_string(),
                LogMetadata::Timestamp => self.build_timestamp(record, &segment.format),
                LogMetadata::CrateName => self.build_crate_name(record, &segment.format),
                LogMetadata::FileName(n) => self.build_file_name(record, &segment.format, *n),
                LogMetadata::FilePath => self.build_file_path(record, &segment.format),
                LogMetadata::ModulePath => self.build_module_path(record, &segment.format),
                LogMetadata::LineNumber => self.build_line_number(record, &segment.format),
                LogMetadata::LogLevel => self.build_log_level(record, &segment.format),
                LogMetadata::Log => self.build_log(record, &segment.format),
                LogMetadata::NestedLogSegments(segments) => {
                    self.build_nested(record, segments, &segment.format)
                }
            };
            result.push_str(&s);
        }

        build_formatted_string(
            &result,
            format,
            0,
            get_log_level_of_record(record),
            format.color,
        )
    }

    fn build_timestamp(&self, record: &Record, format: &LogFormat) -> String {
        let s = match record {
            Record::Defmt(record) if !record.timestamp().is_empty() => record.timestamp(),
            _ => "<time>",
        }
        .to_string();

        build_formatted_string(
            s.as_str(),
            format,
            0,
            get_log_level_of_record(record),
            format.color,
        )
    }

    fn build_log_level(&self, record: &Record, format: &LogFormat) -> String {
        let s = match get_log_level_of_record(record) {
            Some(level) => level.to_string(),
            None => "<lvl>".to_string(),
        };

        let color = format.color.unwrap_or(LogColor::SeverityLevel);

        build_formatted_string(
            s.as_str(),
            format,
            5,
            get_log_level_of_record(record),
            Some(color),
        )
    }

    fn build_file_path(&self, record: &Record, format: &LogFormat) -> String {
        let file_path = match record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        }
        .unwrap_or("<file>");

        build_formatted_string(
            file_path,
            format,
            0,
            get_log_level_of_record(record),
            format.color,
        )
    }

    fn build_file_name(&self, record: &Record, format: &LogFormat, level_of_detail: u8) -> String {
        let file = match record {
            Record::Defmt(record) => record.file(),
            Record::Host(record) => record.file(),
        };

        let s = if let Some(file) = file {
            let path_iter = Path::new(file).iter();
            let number_of_components = path_iter.clone().count();

            let number_of_components_to_join = number_of_components.min(level_of_detail as usize);

            let number_of_elements_to_skip =
                number_of_components.saturating_sub(number_of_components_to_join);
            let s = path_iter
                .skip(number_of_elements_to_skip)
                .take(number_of_components)
                .fold(String::new(), |mut acc, s| {
                    acc.push_str(s.to_str().unwrap_or("<?>"));
                    acc.push('/');
                    acc
                });
            s.strip_suffix('/').unwrap().to_string()
        } else {
            "<file>".to_string()
        };

        build_formatted_string(&s, format, 0, get_log_level_of_record(record), format.color)
    }

    fn build_module_path(&self, record: &Record, format: &LogFormat) -> String {
        let s = match record {
            Record::Defmt(record) => record.module_path(),
            Record::Host(record) => record.module_path(),
        }
        .unwrap_or("<mod path>");

        build_formatted_string(s, format, 0, get_log_level_of_record(record), format.color)
    }

    fn build_crate_name(&self, record: &Record, format: &LogFormat) -> String {
        let module_path = match record {
            Record::Defmt(record) => record.module_path(),
            Record::Host(record) => record.module_path(),
        };

        let s = if let Some(module_path) = module_path {
            let path = module_path.split("::").collect::<Vec<_>>();

            // There need to be at least two elements, the crate and the function
            if path.len() >= 2 {
                path.first().unwrap()
            } else {
                "<crate>"
            }
        } else {
            "<crate>"
        };

        build_formatted_string(s, format, 0, get_log_level_of_record(record), format.color)
    }

    fn build_line_number(&self, record: &Record, format: &LogFormat) -> String {
        let s = match record {
            Record::Defmt(record) => record.line(),
            Record::Host(record) => record.line(),
        }
        .unwrap_or(0)
        .to_string();

        build_formatted_string(
            s.as_str(),
            format,
            4,
            get_log_level_of_record(record),
            format.color,
        )
    }

    fn build_log(&self, record: &Record, format: &LogFormat) -> String {
        let log_level = get_log_level_of_record(record);
        match record {
            Record::Defmt(record) => match color_diff(record.args().to_string()) {
                Ok(s) => s.to_string(),
                Err(s) => build_formatted_string(s.as_str(), format, 0, log_level, format.color),
            },
            Record::Host(record) => record.args().to_string(),
        }
    }
}

fn get_log_level_of_record(record: &Record) -> Option<Level> {
    match record {
        Record::Defmt(record) => record.level(),
        Record::Host(record) => Some(record.level()),
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
    let styled_string_length = s.len();
    let length_without_styles = string_excluding_ansi(&s).len();
    let length_of_ansi_sequences = styled_string_length - length_without_styles;

    let s = apply_color(s, log_color, level);
    let colored_str = apply_styles(s, format.style.as_ref());

    let alignment = format.alignment.unwrap_or(Alignment::Left);
    let width = format.width.unwrap_or(default_width) + length_of_ansi_sequences;
    let padding = format.padding.unwrap_or(Padding::Space);

    let mut result = String::new();
    match (alignment, padding) {
        (Alignment::Left, Padding::Space) => write!(&mut result, "{colored_str:<0$}", width),
        (Alignment::Left, Padding::Zero) => write!(&mut result, "{colored_str:0<0$}", width),
        (Alignment::Center, Padding::Space) => write!(&mut result, "{colored_str:^0$}", width),
        (Alignment::Center, Padding::Zero) => write!(&mut result, "{colored_str:0^0$}", width),
        (Alignment::Right, Padding::Space) => write!(&mut result, "{colored_str:>0$}", width),
        (Alignment::Right, Padding::Zero) => write!(&mut result, "{colored_str:0>0$}", width),
    }
    .expect("Failed to format string: \"{colored_str}\"");
    result
}

fn format_has_timestamp(segments: &[LogSegment]) -> bool {
    for segment in segments {
        match &segment.metadata {
            LogMetadata::Timestamp => return true,
            LogMetadata::NestedLogSegments(s) => {
                if format_has_timestamp(s) {
                    return true;
                }
            }
            _ => continue,
        }
    }
    false
}

/// Returns the given string excluding ANSI control sequences.
fn string_excluding_ansi(s: &str) -> String {
    // Regular expression to match ANSI escape sequences
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();

    // Replace all ANSI sequences with an empty string
    re.replace_all(s, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_left_aligned_styled_string() {
        let format = LogFormat {
            color: Some(LogColor::Color(Color::Green)),
            width: Some(10),
            alignment: Some(Alignment::Left),
            padding: Some(Padding::Space),
            style: Some(vec![Styles::Bold]),
        };

        let s = build_formatted_string("test", &format, 0, None, None);
        let string_without_styles = string_excluding_ansi(&s);
        assert_eq!(string_without_styles, "test      ");
    }

    #[test]
    fn test_right_aligned_styled_string() {
        let format = LogFormat {
            color: Some(LogColor::Color(Color::Green)),
            width: Some(10),
            alignment: Some(Alignment::Right),
            padding: Some(Padding::Space),
            style: Some(vec![Styles::Bold]),
        };

        let s = build_formatted_string("test", &format, 0, None, None);
        let string_without_styles = string_excluding_ansi(&s);
        assert_eq!(string_without_styles, "      test");
    }
}
