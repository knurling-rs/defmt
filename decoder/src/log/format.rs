use nom::{
    branch::alt,
    bytes::complete::{take_till1, take_while},
    character::complete::{char, digit1, one_of},
    combinator::{map, map_res, opt},
    multi::{many0, separated_list1},
    sequence::delimited,
    IResult, Parser,
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

fn parse_log_segment(input: &str) -> IResult<&str, LogSegment, ()> {
    let (input, output) = separated_list1(
        char(':'),
        alt((
            parse_metadata,
            parse_color,
            parse_style,
            parse_width_and_alignment,
        )),
    )(input)?;

    let mut metadata = None;
    let mut color = None;
    let mut style = None;
    let mut width_and_alignment = None;
    for item in output {
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
            _ => return Err(nom::Err::Failure(())),
        }
    }

    let Some(metadata) = metadata else {
        return Err(nom::Err::Failure(()));
    };

    let (width, alignment) = width_and_alignment
        .map(|(w, a)| (Some(w), a))
        .unwrap_or((None, None));

    let log_segment = LogSegment {
        metadata,
        format: LogFormat {
            color,
            style,
            width,
            alignment,
        },
    };

    Ok((input, log_segment))
}

fn parse_argument(input: &str) -> IResult<&str, LogSegment, ()> {
    let mut parse_enclosed = delimited(char('{'), parse_log_segment, char('}'));
    parse_enclosed.parse(input)
}

fn parse_string_segment(input: &str) -> IResult<&str, LogSegment, ()> {
    map(take_till1(|c| c == '{'), |s: &str| {
        LogSegment::new(LogMetadata::String(s.to_string()))
    })
    .parse(input)
}

pub(super) fn parse(input: &str) -> Result<Vec<LogSegment>, String> {
    let mut parse_all = many0(alt((parse_argument, parse_string_segment)));

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
        let result = parse_argument("{t}");
        assert_eq!(result, Ok(("", LogSegment::new(LogMetadata::Timestamp))));
    }

    #[test]
    fn test_parse_argument_with_color() {
        let result = parse_log_segment("t:werror");
        let expected_output =
            LogSegment::new(LogMetadata::Timestamp).with_color(LogColor::WarnError);
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_width_first() {
        let result = parse_argument("{t:>8:white}");
        let expected_output = LogSegment::new(LogMetadata::Timestamp)
            .with_width(8)
            .with_alignment(Alignment::Right)
            .with_color(LogColor::Color(colored::Color::White));
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_argument_with_extra_format_parameters_color_first() {
        let result = parse_argument("{f:werror:<25}");
        let expected_output = LogSegment::new(LogMetadata::FileName)
            .with_width(25)
            .with_alignment(Alignment::Left)
            .with_color(LogColor::WarnError);
        assert_eq!(result, Ok(("", expected_output)));
    }

    #[test]
    fn test_parse_invalid_argument() {
        let result = parse_argument("{foo}");
        assert_eq!(result, Result::Err(nom::Err::Error(())));
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
}
