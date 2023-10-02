//! Formatting for `defmt` log frames.
//!
//! This module contains the parsers for the `defmt` custom log formatting strings.
//!
//! # Format strings
//!
//! A format string takes a set of [format specifiers] written
//! in the way a log should be printed by the logger.
//!
//! ## Basics
//!
//! Format strings allow the customization of how the logger prints logs.
//!
//! The following log will be used as reference in the examples below:
//!
//! ```ignore
//! defmt::error!("hello");
//! ```
//!
//! The simplest format string is `"{s}"`. This prints the log and nothing else:
//!
//! ```text
//! hello
//! ```
//!
//! Arbitrary text can be added to the format string, which will be printed as specified with each log.
//! For example, `"Log: {s}"`:
//!
//! ```text
//! Log: hello
//! ```
//!
//! Multiple specifiers can be included within a format string, in any order. For example `"[{L}] {s}"` prints:
//!
//! ```text
//! [ERROR] hello
//! ```
//!
//! ## Customizing log segments
//!
//! The way a format specifier is printed can be customized by providing additional, optional [format parameters].
//!
//! Format parameters are provided by adding the parameters after the format specifier, separated by colons (`:`),
//! like this: `"{L:bold:5} {f:white:<10} {s}"`.
//!
//! ### Color
//!
//! A log segment can be specified to be colored by providing a color in the format parameters.
//!
//! There are three different options for coloring a log segment:
//! - a string that can be parsed by the FromStr implementation of [colored::Color].
//! - `severity` colors the log segment using the predefined color for the log level of log.
//! - `werror` is similar to `severity`, but it only applies the color if the log level is WARN or ERROR.
//!
//! Only one coloring option can be provided in format parameters for a given format specifier.
//!
//! ### Styles
//!
//! A log segment can be specified to be printed with a given style by providing a style in the format parameters.
//!
//! The style specifier must be one of the following strings:
//! - `bold`
//! - `italic`
//! - `underline`
//! - `strike`
//! - `dimmed`
//!
//! Multiple styles can be applied to a single format specifier, but they must not be repeated, i.e.
//! `"{s:bold:underline:italic}"` is allowed, but `"{s:bold:bold}"` isn't.
//!
//! ### Width and alignment
//!
//! A log segment can be specified to be printed with a given minimum width and alignment by providing a format parameter.
//!
//! The alignment can be specified to be left (`<`), right (`>`), or center-aligned (`^`).
//! If no alignment is specified, left alignment is used by default.
//!
//! The minimum width is specified after the alignment.
//! For example, "{L} {f:>10}: {s}" is printed as follows:
//!
//! ```text
//! [ERROR]    main.rs: hello
//! ```
//!
//! The log level format specifier is printed with a minimum width of 5 by default.
//!
//! ## Nested formatting
//!
//! Log segments can be grouped and formatted together by nesting formats. Format parameters for the grouped log segments
//! must be provided after the group, separated by `%`.
//!
//! Nested formats allow for more intricate formatting. For example, `"{[{L:bold}]%underline} {s}"` prints
//!
//! ```text
//! [ERROR] hello
//! ```
//!
//! where `ERROR` is formatted bold, and `[ERROR]` is underlined.
//!
//! Formats can be nested several levels. This provides a great level of flexibility to customize the logger formatting.
//! For example, the width and alignment of a group of log segments can be specified with nested formats.
//! `"{{[{L}]%bold} {f:>20}:%<35} {s}"` prints:
//!
//! ```text
//! [ERROR]              main.rs:       hello
//! ```
//!
//! ## Restrictions
//!
//! - Format strings *must* include the `{s}` format specifier (log specifier).
//! - At the moment it is not possible to escape curly brackets (i.e. `{`, `}`)
//!   in the format string, therefore curly brackets cannot be printed as part
//!   of the logger format.
//! - The same restriction exists for the `%` character.
//!
//! [format specifiers]: LogMetadata
//! [format parameters]: LogFormat

use nom::{
    branch::alt,
    bytes::complete::{take_till1, take_while},
    character::complete::{char, digit1, one_of},
    combinator::{map, map_res, opt},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded},
    IResult, Parser,
};

use std::str::FromStr;

/// Representation of what a [LogSegment] can be.
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub(super) enum LogMetadata {
    /// `{f}` format specifier.
    ///
    /// For a file "src/foo/bar.rs", this option prints "bar.rs".
    FileName,

    /// `{F}` format specifier.
    ///
    /// For a file "src/foo/bar.rs", this option prints "src/foo/bar.rs".
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
    TimestampMs,

    /// `{T}` format specifier.
    ///
    /// Prints the timestamp at which something was logged, but printing it
    /// in Unix style (hh:mm:ss.milliseconds).
    /// For a log printed with timestamp 123456 ms, this prints "00:02:03.456".
    TimestampUnix,

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

    pub fn is_timestamp(&self) -> bool {
        matches!(
            self,
            LogMetadata::TimestampMs | LogMetadata::TimestampUnix
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

#[derive(Debug, PartialEq, Clone)]
pub(super) struct LogFormat {
    pub(super) width: Option<usize>,
    pub(super) padding: Option<Padding>,
    pub(super) color: Option<LogColor>,
    pub(super) style: Option<Vec<colored::Styles>>,
    pub(super) alignment: Option<Alignment>,
}

#[derive(Debug, PartialEq, Clone)]
enum IntermediateOutput {
    Metadata(LogMetadata),
    WidthAndAlignment((usize, Padding, Option<Alignment>)),
    Color(LogColor),
    Style(colored::Styles),
    NestedLogSegment(LogSegment),
}

impl LogSegment {
    pub(super) const fn new(metadata: LogMetadata) -> Self {
        Self {
            metadata,
            format: LogFormat {
                color: None,
                style: None,
                padding: None,
                width: None,
                alignment: None,
            },
        }
    }

    #[cfg(test)]
    const fn with_color(mut self, color: LogColor) -> Self {
        self.format.color = Some(color);
        self
    }

    #[cfg(test)]
    fn with_style(mut self, style: colored::Styles) -> Self {
        let mut styles = self.format.style.unwrap_or_default();
        styles.push(style);
        self.format.style = Some(styles);
        self
    }

    #[cfg(test)]
    const fn with_width(mut self, width: usize) -> Self {
        self.format.width = Some(width);
        self
    }

    #[cfg(test)]
    const fn with_padding(mut self, padding: Padding) -> Self {
        self.format.padding = Some(padding);
        self
    }

    #[cfg(test)]
    const fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.format.alignment = Some(alignment);
        self
    }
}

/// This function is taken as-is from the parse-hyperlinks crate
/// https://docs.rs/parse-hyperlinks/0.9.3/src/parse_hyperlinks/lib.rs.html#24-68
/// There is an open issue in nom to include this parser in nom 8.0
/// https://github.com/rust-bakery/nom/issues/1253
pub fn take_until_unbalanced(
    opening_bracket: char,
    closing_bracket: char,
) -> impl Fn(&str) -> IResult<&str, &str, ()> {
    move |i: &str| {
        let mut index = 0;
        let mut bracket_counter = 0;
        while let Some(n) = &i[index..].find(&[opening_bracket, closing_bracket, '\\'][..]) {
            index += n;
            let mut it = i[index..].chars();
            match it.next().unwrap_or_default() {
                c if c == '\\' => {
                    // Skip the escape char `\`.
                    index += '\\'.len_utf8();
                    // Skip also the following char.
                    let c = it.next().unwrap_or_default();
                    index += c.len_utf8();
                }
                c if c == opening_bracket => {
                    bracket_counter += 1;
                    index += opening_bracket.len_utf8();
                }
                c if c == closing_bracket => {
                    // Closing bracket.
                    bracket_counter -= 1;
                    index += closing_bracket.len_utf8();
                }
                // Can not happen.
                _ => unreachable!(),
            };
            // We found the unmatched closing bracket.
            if bracket_counter == -1 {
                // We do not consume it.
                index -= closing_bracket.len_utf8();
                return Ok((&i[index..], &i[0..index]));
            };
        }

        if bracket_counter == 0 {
            Ok(("", i))
        } else {
            Err(nom::Err::Error(()))
        }
    }
}

fn parse_metadata(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let mut parse_type = map_res(take_while(char::is_alphanumeric), move |s| {
        let metadata = match s {
            "f" => LogMetadata::FileName,
            "F" => LogMetadata::FilePath,
            "l" => LogMetadata::LineNumber,
            "s" => LogMetadata::Log,
            "L" => LogMetadata::LogLevel,
            "m" => LogMetadata::ModulePath,
            "t" => LogMetadata::TimestampMs,
            "T" => LogMetadata::TimestampUnix,
            _ => return Err(()),
        };
        Ok(IntermediateOutput::Metadata(metadata))
    });

    parse_type.parse(input)
}

fn parse_color(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let mut parse_type = map_res(take_while(char::is_alphabetic), move |s| {
        let color = match s {
            "severity" => LogColor::SeverityLevel,
            "werror" => LogColor::WarnError,
            s => match colored::Color::from_str(s) {
                Ok(c) => LogColor::Color(c),
                Err(()) => return Err(()),
            },
        };
        Ok(IntermediateOutput::Color(color))
    });

    parse_type.parse(input)
}

fn parse_style(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let mut parse_type = map_res(take_while(char::is_alphabetic), move |s| {
        let style = match s {
            "bold" => colored::Styles::Bold,
            "italic" => colored::Styles::Italic,
            "underline" => colored::Styles::Underline,
            "strike" => colored::Styles::Strikethrough,
            "dimmed" => colored::Styles::Dimmed,
            _ => return Err(()),
        };
        Ok(IntermediateOutput::Style(style))
    });

    parse_type.parse(input)
}

fn parse_width_and_alignment(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let (input, alignment) = opt(map_res(one_of("<^>"), move |c| match c {
        '^' => Ok(Alignment::Center),
        '<' => Ok(Alignment::Left),
        '>' => Ok(Alignment::Right),
        _ => Err(()),
    }))(input)?;

    let (input, width) = digit1.parse(input)?;

    let padding = if width.starts_with('0') {
        Padding::Zero
    } else {
        Padding::Space
    };

    let Ok(width) = width.parse::<usize>() else {
        return Err(nom::Err::Error(()));
    };

    Ok((
        input,
        IntermediateOutput::WidthAndAlignment((width, padding, alignment)),
    ))
}

fn parse_format<const FAIL_ON_ERR: bool>(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let result = alt((parse_color, parse_style, parse_width_and_alignment)).parse(input);

    if !FAIL_ON_ERR {
        result
    } else {
        match result {
            Ok(r) => Ok(r),
            Err(_) => Err(nom::Err::Failure(())),
        }
    }
}

fn build_log_segment<const NEST: bool>(
    intermediate_output: Vec<IntermediateOutput>,
) -> Result<LogSegment, nom::Err<()>> {
    let mut metadata = None;
    let mut color = None;
    let mut style = None;
    let mut width_and_alignment = None;
    let mut nested_segments = None;
    for item in intermediate_output {
        match item {
            IntermediateOutput::Metadata(m) if metadata.is_none() => metadata = Some(m),
            IntermediateOutput::Color(c) if color.is_none() => color = Some(c),
            IntermediateOutput::Style(s) => {
                let mut styles: Vec<colored::Styles> = style.unwrap_or_default();

                // A format with repeated style specifiers is not valid
                if styles.contains(&s) {
                    return Err(nom::Err::Failure(()));
                }

                styles.push(s);
                style = Some(styles);
            }
            IntermediateOutput::WidthAndAlignment(w) if width_and_alignment.is_none() => {
                width_and_alignment = Some(w)
            }
            IntermediateOutput::NestedLogSegment(s) => {
                let mut segments: Vec<LogSegment> = nested_segments.unwrap_or_default();
                segments.push(s);
                nested_segments = Some(segments);
            }
            _ => return Err(nom::Err::Failure(())),
        }
    }

    if NEST {
        let Some(nested_segments) = nested_segments else {
            return Err(nom::Err::Failure(()));
        };

        // A nested segment must have at least a valid specifier such as {t} or {f},
        // it isn't allowed to have {foo} and consider foo a string segment.
        let has_metadata_specifier = nested_segments
            .iter()
            .any(|segment| segment.metadata.is_metadata_specifier());
        if !has_metadata_specifier {
            return Err(nom::Err::Failure(()));
        }
        metadata = Some(LogMetadata::NestedLogSegments(nested_segments));
    } else {
        // We either have nested segments, or a metadata specifier, we can't have both
        // This means we either have:
        //  metadata specifier: {L:underline}
        //  nested segments specifier: {[{L:<5:bold}]%underline}
        if metadata.is_some() && nested_segments.is_some() {
            return Err(nom::Err::Failure(()));
        }

        // If we have a nested segment there must be exactly one,
        // otherwise there's something weird going on
        if let Some(segments) = nested_segments {
            if segments.len() == 1 {
                return Ok(segments[0].clone());
            } else {
                return Err(nom::Err::Failure(()));
            }
        }
    }

    let Some(metadata) = metadata else {
        return Err(nom::Err::Failure(()));
    };

    let (width, padding, alignment) = width_and_alignment
        .map(|(w, p, a)| (Some(w), Some(p), a))
        .unwrap_or((None, None, None));

    Ok(LogSegment {
        metadata,
        format: LogFormat {
            color,
            style,
            width,
            padding,
            alignment,
        },
    })
}

fn parse_log_segment<const NEST: bool>(input: &str) -> IResult<&str, LogSegment, ()> {
    let (input, output) = if !NEST {
        separated_list1(
            char(':'),
            alt((parse_metadata, parse_format::<false>, parse_nested::<true>)),
        )(input)
    } else {
        let parse_nested_argument =
            separated_list1(char(':'), alt((parse_metadata, parse_format::<false>)));

        let parse_nested_log_segment = map_res(parse_nested_argument, |result| {
            let log_segment = build_log_segment::<false>(result)?;
            Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(
                log_segment,
            ))
        });

        separated_list1(
            char('%'),
            alt((
                parse_nested_log_segment,
                parse_format::<false>,
                parse_nested::<false>,
            )),
        )(input)
    }?;

    let log_segment = build_log_segment::<false>(output)?;
    Ok((input, log_segment))
}

fn parse_argument<const NEST: bool>(input: &str) -> IResult<&str, LogSegment, ()> {
    let take_between_matching_brackets =
        delimited(char('{'), take_until_unbalanced('{', '}'), char('}'));

    take_between_matching_brackets
        .and_then(parse_log_segment::<NEST>)
        .parse(input)
}

fn parse_string_segment(input: &str) -> IResult<&str, LogSegment, ()> {
    map(take_till1(|c| c == '{' || c == '%'), |s: &str| {
        LogSegment::new(LogMetadata::String(s.to_string()))
    })
    .parse(input)
}

fn parse_nested<const NEST: bool>(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let parse_nested_argument = map_res(parse_argument::<NEST>, |result| {
        Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(result))
    });
    let parse_nested_string_segment = map_res(parse_string_segment, |result| {
        Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(result))
    });
    let parse_nested_format = preceded(char('%'), parse_format::<true>);
    let mut parse_all = many0(alt((
        parse_nested_argument,
        parse_nested_string_segment,
        parse_nested_format,
    )));

    let (new_input, output) = parse_all(input)?;
    let log_segment = build_log_segment::<true>(output)?;
    Ok((new_input, IntermediateOutput::NestedLogSegment(log_segment)))
}

fn format_contains_log_specifier(segments: &[LogSegment]) -> bool {
    for segment in segments {
        match &segment.metadata {
            LogMetadata::Log => return true,
            LogMetadata::NestedLogSegments(s) => {
                if format_contains_log_specifier(s) {
                    return true;
                }
            }
            _ => continue,
        }
    }
    false
}

pub(super) fn parse(input: &str) -> Result<Vec<LogSegment>, String> {
    let mut parse_all = many0(alt((parse_argument::<false>, parse_string_segment)));

    let result = parse_all(input)
        .map(|(_, output)| output)
        .map_err(|e| e.to_string())?;

    if !format_contains_log_specifier(&result) {
        return Err("log format must contain a `{s}` format specifier".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_template() {
        let log_template = "{t} [{L}] {s}\n└─ {m} @ {F}:{l}";

        let expected_output = vec![
            LogSegment::new(LogMetadata::TimestampMs),
            LogSegment::new(LogMetadata::String(" [".to_string())),
            LogSegment::new(LogMetadata::LogLevel),
            LogSegment::new(LogMetadata::String("] ".to_string())),
            LogSegment::new(LogMetadata::Log),
            LogSegment::new(LogMetadata::String("\n└─ ".to_string())),
            LogSegment::new(LogMetadata::ModulePath),
            LogSegment::new(LogMetadata::String(" @ ".to_string())),
            LogSegment::new(LogMetadata::FilePath),
            LogSegment::new(LogMetadata::String(":".to_string())),
            LogSegment::new(LogMetadata::LineNumber),
        ];

        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_format_without_log() {
        let result = parse("{t}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_segment() {
        let result = parse_string_segment("Log: {t}");
        let (input, output) = result.unwrap();
        assert_eq!(input, "{t}");
        assert_eq!(
            output,
            LogSegment::new(LogMetadata::String("Log: ".to_string()))
        );
    }

    #[test]
    fn test_parse_empty_string_segment() {
        let result = parse_string_segment("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_timestamp_argument() {
        let result = parse_argument::<false>("{t}");
        assert_eq!(result, Ok(("", LogSegment::new(LogMetadata::TimestampMs))));
    }

    #[test]
    fn test_parse_argument_with_color() {
        let result = parse_log_segment::<false>("t:werror");
        let expected_output =
            LogSegment::new(LogMetadata::TimestampMs).with_color(LogColor::WarnError);
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_width_first() {
        let result = parse_argument::<false>("{t:>8:white}");
        let expected_output = LogSegment::new(LogMetadata::TimestampMs)
            .with_width(8)
            .with_padding(Padding::Space)
            .with_alignment(Alignment::Right)
            .with_color(LogColor::Color(colored::Color::White));
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_color_first() {
        let result = parse_argument::<false>("{f:werror:<25}");
        let expected_output = LogSegment::new(LogMetadata::FileName)
            .with_width(25)
            .with_padding(Padding::Space)
            .with_alignment(Alignment::Left)
            .with_color(LogColor::WarnError);
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_invalid_argument() {
        let result = parse_argument::<false>("{foo}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_width_no_alignment() {
        let result = parse_width_and_alignment("12");
        assert_eq!(
            result,
            Ok(("", IntermediateOutput::WidthAndAlignment((12, Padding::Space, None))))
        );
    }

    #[test]
    fn test_parse_width_with_zero_padding_no_alignment() {
        let result = parse_width_and_alignment("012");
        assert_eq!(
            result,
            Ok(("", IntermediateOutput::WidthAndAlignment((12, Padding::Zero, None))))
        );
    }

    #[test]
    fn test_parse_width_and_alignment() {
        let result = parse_width_and_alignment(">12");
        assert_eq!(
            result,
            Ok((
                "",
                IntermediateOutput::WidthAndAlignment((12, Padding::Space, Some(Alignment::Right)))
            ))
        );
    }

    #[test]
    fn test_parse_width_zero_padding_and_alignment() {
        let result = parse_width_and_alignment(">012");
        assert_eq!(
            result,
            Ok((
                "",
                IntermediateOutput::WidthAndAlignment((12, Padding::Zero, Some(Alignment::Right)))
            ))
        );
    }

    #[test]
    fn test_parse_color() {
        let result = parse_color("blue");
        assert_eq!(
            result,
            Ok((
                "",
                IntermediateOutput::Color(LogColor::Color(colored::Color::Blue))
            ))
        );
    }

    #[test]
    fn test_parse_log_template_with_color_style_width_and_alignment() {
        let log_template = "T{t:>8} [{L:severity:bold}] {f:white:underline}:{l:white:3} {s:werror}";

        let expected_output = vec![
            LogSegment::new(LogMetadata::String("T".to_string())),
            LogSegment::new(LogMetadata::TimestampMs)
                .with_width(8)
                .with_padding(Padding::Space)
                .with_alignment(Alignment::Right),
            LogSegment::new(LogMetadata::String(" [".to_string())),
            LogSegment::new(LogMetadata::LogLevel)
                .with_color(LogColor::SeverityLevel)
                .with_style(colored::Styles::Bold),
            LogSegment::new(LogMetadata::String("] ".to_string())),
            LogSegment::new(LogMetadata::FileName)
                .with_color(LogColor::Color(colored::Color::White))
                .with_style(colored::Styles::Underline),
            LogSegment::new(LogMetadata::String(":".to_string())),
            LogSegment::new(LogMetadata::LineNumber)
                .with_color(LogColor::Color(colored::Color::White))
                .with_width(3)
                .with_padding(Padding::Space),
            LogSegment::new(LogMetadata::String(" ".to_string())),
            LogSegment::new(LogMetadata::Log).with_color(LogColor::WarnError),
        ];

        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_log_with_multiple_different_styles() {
        let log_template = "{s:bold:underline:italic:dimmed}";

        let expected_output = vec![LogSegment::new(LogMetadata::Log)
            .with_style(colored::Styles::Bold)
            .with_style(colored::Styles::Underline)
            .with_style(colored::Styles::Italic)
            .with_style(colored::Styles::Dimmed)];

        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_log_with_repeated_styles() {
        let log_template = "{s:bold:bold}";
        let result = parse(log_template);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_single_nested_format() {
        let log_template = "{[{L:<5:bold}]%underline%italic} {s}";
        let expected_output = vec![
            LogSegment::new(LogMetadata::NestedLogSegments(vec![
                LogSegment::new(LogMetadata::String("[".to_string())),
                LogSegment::new(LogMetadata::LogLevel)
                    .with_alignment(Alignment::Left)
                    .with_width(5)
                    .with_padding(Padding::Space)
                    .with_style(colored::Styles::Bold),
                LogSegment::new(LogMetadata::String("]".to_string())),
            ]))
            .with_style(colored::Styles::Underline)
            .with_style(colored::Styles::Italic),
            LogSegment::new(LogMetadata::String(" ".to_string())),
            LogSegment::new(LogMetadata::Log),
        ];
        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_single_nested_format_with_bad_specifier() {
        let log_template = "{[{L:<5:bold}]%bad%underline} {s}";
        let result = parse(log_template);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_double_nested_format() {
        let log_template = "{{[{L:<5}]%bold} {f:>20}:%<30} {s}";
        let expected_output = vec![
            LogSegment::new(LogMetadata::NestedLogSegments(vec![
                LogSegment::new(LogMetadata::NestedLogSegments(vec![
                    LogSegment::new(LogMetadata::String("[".to_string())),
                    LogSegment::new(LogMetadata::LogLevel)
                        .with_alignment(Alignment::Left)
                        .with_width(5)
                        .with_padding(Padding::Space),
                    LogSegment::new(LogMetadata::String("]".to_string())),
                ]))
                .with_style(colored::Styles::Bold),
                LogSegment::new(LogMetadata::String(" ".to_string())),
                LogSegment::new(LogMetadata::FileName)
                    .with_alignment(Alignment::Right)
                    .with_width(20)
                    .with_padding(Padding::Space),
                LogSegment::new(LogMetadata::String(":".to_string())),
            ]))
            .with_alignment(Alignment::Left)
            .with_width(30)
            .with_padding(Padding::Space),
            LogSegment::new(LogMetadata::String(" ".to_string())),
            LogSegment::new(LogMetadata::Log),
        ];
        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_triple_nested_format() {
        let log_template = "{{{[{L:<5}]%bold} {f:>20}:%<30} {s}%werror}";
        let expected_output = vec![LogSegment::new(LogMetadata::NestedLogSegments(vec![
            LogSegment::new(LogMetadata::NestedLogSegments(vec![
                LogSegment::new(LogMetadata::NestedLogSegments(vec![
                    LogSegment::new(LogMetadata::String("[".to_string())),
                    LogSegment::new(LogMetadata::LogLevel)
                        .with_alignment(Alignment::Left)
                        .with_width(5)
                        .with_padding(Padding::Space),
                    LogSegment::new(LogMetadata::String("]".to_string())),
                ]))
                .with_style(colored::Styles::Bold),
                LogSegment::new(LogMetadata::String(" ".to_string())),
                LogSegment::new(LogMetadata::FileName)
                    .with_alignment(Alignment::Right)
                    .with_width(20)
                    .with_padding(Padding::Space),
                LogSegment::new(LogMetadata::String(":".to_string())),
            ]))
            .with_alignment(Alignment::Left)
            .with_width(30)
            .with_padding(Padding::Space),
            LogSegment::new(LogMetadata::String(" ".to_string())),
            LogSegment::new(LogMetadata::Log),
        ]))
        .with_color(LogColor::WarnError)];
        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }
}
