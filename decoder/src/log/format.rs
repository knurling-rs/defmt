use nom::{
    branch::alt,
    bytes::complete::{take_till1, take_while},
    character::complete::{char, digit1, one_of},
    combinator::{map, map_res, opt},
    multi::{many0, separated_list1},
    sequence::{delimited, preceded},
    error::{Error, ErrorKind, ParseError},
    Err, IResult, Parser,
};

use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub(super) enum LogMetadata {
    FileName,
    FilePath,
    LineNumber,
    Log,
    LogLevel,
    ModulePath,
    String(String),
    Timestamp,
    NestedLogSegments(Vec<LogSegment>),
}

impl LogMetadata {
    /// Checks whether this `LogMetadata` came from a specifier such as
    /// {t}, {f}, etc.
    fn is_metadata_specifier(&self) -> bool {
        match self {
            LogMetadata::String(_) | LogMetadata::NestedLogSegments(_) => false,
            _ => true,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum LogColor {
    /// User-defined color
    Color(colored::Color),

    /// Color matching the default color for the log level
    SeverityLevel,

    /// Color matching the default color for the log level,
    /// but only if the log level is WARN or ERROR
    WarnError,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Alignment {
    Center,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Clone)]
pub(super) struct LogSegment {
    pub(super) metadata: LogMetadata,
    pub(super) format: LogFormat,
}

#[derive(Debug, PartialEq, Clone)]
pub(super) struct LogFormat {
    pub(super) width: Option<usize>,
    pub(super) color: Option<LogColor>,
    pub(super) style: Option<Vec<colored::Styles>>,
    pub(super) alignment: Option<Alignment>,
}

#[derive(Debug, PartialEq, Clone)]
enum IntermediateOutput {
    Metadata(LogMetadata),
    WidthAndAlignment((usize, Option<Alignment>)),
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
    let mut parse_type = map_res(take_while(char::is_alphabetic), move |s| {
        let metadata = match s {
            "f" => LogMetadata::FileName,
            "F" => LogMetadata::FilePath,
            "l" => LogMetadata::LineNumber,
            "s" => LogMetadata::Log,
            "L" => LogMetadata::LogLevel,
            "m" => LogMetadata::ModulePath,
            "t" => LogMetadata::Timestamp,
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

    let (input, width) = map_res(digit1, move |s: &str| s.parse::<usize>())(input)?;

    Ok((
        input,
        IntermediateOutput::WidthAndAlignment((width, alignment)),
    ))
}

fn parse_format<const FAIL_ON_ERR: bool>(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let result = alt((
        parse_color,
        parse_style,
        parse_width_and_alignment,
    )).parse(input);

    if !FAIL_ON_ERR {
        result
    } else {
        match result {
            Ok(r) => Ok(r),
            Err(_) => Err(nom::Err::Failure(())),
        }
    }
}

fn build_log_segment<const NEST: bool>(intermediate_output: Vec<IntermediateOutput>) -> Result<LogSegment, nom::Err<()>> {
    let mut metadata = None;
    let mut color = None;
    let mut style = None;
    let mut width_and_alignment = None;
    let mut nested_segments = None;
    for item in intermediate_output {
        match item {
            IntermediateOutput::Metadata(m) if metadata.is_none() => {
                metadata = Some(m)
            },
            IntermediateOutput::Color(c) if color.is_none() => {
                color = Some(c)
            },
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
            _ => {
                return Err(nom::Err::Failure(()))
            },
        }
    }

    if NEST {
        let Some(nested_segments) = nested_segments else {
            return Err(nom::Err::Failure(()));
        };

        // A nested segment must have at least a valid specifier such as {t} or {f},
        // it isn't allowed to have {foo} and consider foo a string segment.
        let has_metadata_specifier = nested_segments.iter().any(|segment| segment.metadata.is_metadata_specifier());
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
            if segments.iter().count() == 1 {
                return Ok(segments[0].clone());
            } else {
                return Err(nom::Err::Failure(()));
            }
        }
    }

    let Some(metadata) = metadata else {
        return Err(nom::Err::Failure(()));
    };

    let (width, alignment) = width_and_alignment
        .map(|(w, a)| (Some(w), a))
        .unwrap_or((None, None));

    Ok(LogSegment {
        metadata,
        format: LogFormat {
            color,
            style,
            width,
            alignment,
        },
    })
}


fn parse_log_segment<const NEST: bool>(input: &str) -> IResult<&str, LogSegment, ()> {
    let (input, output) = if !NEST {
        separated_list1(
            char(':'),
            alt((
                parse_metadata,
                parse_format::<false>,
                parse_nested::<true>,
            )),
        )(input)
    } else {
        let parse_nested_argument = separated_list1(
            char(':'),
            alt((parse_metadata, parse_format::<false>)),
        );

        let parse_nested_log_segment = map_res(parse_nested_argument, |result| {
            let log_segment = build_log_segment::<false>(result)?;
            Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(log_segment))
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
    let take_between_matching_brackets = delimited(
        char('{'), 
        take_until_unbalanced('{', '}'), 
        char('}')
    );
    
    take_between_matching_brackets.and_then(parse_log_segment::<NEST>).parse(input)
}

fn parse_string_segment(input: &str) -> IResult<&str, LogSegment, ()> {
    map(take_till1(|c| c == '{' || c == '%'), |s: &str| {
        LogSegment::new(LogMetadata::String(s.to_string()))
    })
    .parse(input)
}

fn parse_nested<const NEST: bool>(input: &str) -> IResult<&str, IntermediateOutput, ()> {
    let parse_nested_argument = map_res(parse_argument::<NEST>, |result| Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(result)));
    let parse_nested_string_segment = map_res(parse_string_segment, |result| Ok::<IntermediateOutput, nom::Err<()>>(IntermediateOutput::NestedLogSegment(result)));
    let parse_nested_format = preceded(char('%'), parse_format::<true>);
    let mut parse_all = many0(alt((parse_nested_argument, parse_nested_string_segment, parse_nested_format)));

    let (new_input, output) = parse_all(input)?;
    let log_segment = build_log_segment::<true>(output)?;
    Ok((new_input, IntermediateOutput::NestedLogSegment(log_segment)))
}

pub(super) fn parse(input: &str) -> Result<Vec<LogSegment>, String> {
    let mut parse_all = many0(alt((parse_argument::<false>, parse_string_segment)));

    parse_all(input)
        .map(|(_, output)| output)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_template() {
        let log_template = "{t} [{L}] {s}\n└─ {m} @ {F}:{l}";

        let expected_output = vec![
            LogSegment::new(LogMetadata::Timestamp),
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
        assert_eq!(result, Ok(("", LogSegment::new(LogMetadata::Timestamp))));
    }

    #[test]
    fn test_parse_argument_with_color() {
        let result = parse_log_segment::<false>("t:werror");
        let expected_output =
            LogSegment::new(LogMetadata::Timestamp).with_color(LogColor::WarnError);
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_width_first() {
        let result = parse_argument::<false>("{t:>8:white}");
        let expected_output = LogSegment::new(LogMetadata::Timestamp)
            .with_width(8)
            .with_alignment(Alignment::Right)
            .with_color(LogColor::Color(colored::Color::White));
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_color_first() {
        let result = parse_argument::<false>("{f:werror:<25}");
        let expected_output = LogSegment::new(LogMetadata::FileName)
            .with_width(25)
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
            Ok(("", IntermediateOutput::WidthAndAlignment((12, None))))
        );
    }

    #[test]
    fn test_parse_width_and_alignment() {
        let result = parse_width_and_alignment(">12");
        assert_eq!(
            result,
            Ok((
                "",
                IntermediateOutput::WidthAndAlignment((12, Some(Alignment::Right)))
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
            LogSegment::new(LogMetadata::Timestamp)
                .with_width(8)
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
                .with_width(3),
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
            LogSegment::new(LogMetadata::NestedLogSegments(
                vec![
                    LogSegment::new(LogMetadata::String("[".to_string())),
                    LogSegment::new(LogMetadata::LogLevel)
                        .with_alignment(Alignment::Left)
                        .with_width(5)
                        .with_style(colored::Styles::Bold),
                    LogSegment::new(LogMetadata::String("]".to_string())),
                ]
            ))
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
            LogSegment::new(LogMetadata::NestedLogSegments(
                vec![
                    LogSegment::new(LogMetadata::NestedLogSegments(
                        vec![
                            LogSegment::new(LogMetadata::String("[".to_string())),
                            LogSegment::new(LogMetadata::LogLevel)
                                .with_alignment(Alignment::Left)
                                .with_width(5),
                            LogSegment::new(LogMetadata::String("]".to_string())),
                        ]
                    )).with_style(colored::Styles::Bold),
                    LogSegment::new(LogMetadata::String(" ".to_string())),
                    LogSegment::new(LogMetadata::FileName)
                        .with_alignment(Alignment::Right)
                        .with_width(20),
                    LogSegment::new(LogMetadata::String(":".to_string())),
                ]
            ))
            .with_alignment(Alignment::Left)
            .with_width(30),
            LogSegment::new(LogMetadata::String(" ".to_string())),
            LogSegment::new(LogMetadata::Log),
        ];
        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }
}
