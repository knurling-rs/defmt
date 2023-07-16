use nom::branch::alt;
use nom::bytes::complete::{take, take_till1};
use nom::character::complete::char;
use nom::combinator::{map, map_res};
use nom::error::{FromExternalError, ParseError};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;
use nom::Parser;

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum LogSegment {
    String(String),
    Timestamp,
    FileName,
    FilePath,
    ModulePath,
    LineNumber,
    LogLevel,
    Log,
}

#[derive(Debug, PartialEq)]
pub struct InvalidArgument;

fn parse_argument<'a, E>(input: &'a str) -> IResult<&'a str, LogSegment, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, InvalidArgument>,
{
    let parse_enclosed = delimited(char('{'), take(1u32), char('}'));
    let mut parse_type = map_res(parse_enclosed, move |s| match s {
        "t" => Ok(LogSegment::Timestamp),
        "f" => Ok(LogSegment::FileName),
        "F" => Ok(LogSegment::FilePath),
        "m" => Ok(LogSegment::ModulePath),
        "l" => Ok(LogSegment::LineNumber),
        "L" => Ok(LogSegment::LogLevel),
        "s" => Ok(LogSegment::Log),
        _ => Err(InvalidArgument),
    });

    parse_type.parse(input)
}

fn parse_string_segment<'a, E>(input: &'a str) -> IResult<&'a str, LogSegment, E>
where
    E: ParseError<&'a str>,
{
    map(take_till1(|c| c == '{'), |s: &str| {
        LogSegment::String(s.to_string())
    })
    .parse(input)
}

pub fn parse(input: &str) -> Result<Vec<LogSegment>, String> {
    let mut parse_all = many0(alt((parse_argument::<()>, parse_string_segment)));

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
            LogSegment::Timestamp,
            LogSegment::String(" [".to_string()),
            LogSegment::LogLevel,
            LogSegment::String("] ".to_string()),
            LogSegment::Log,
            LogSegment::String("\n└─ ".to_string()),
            LogSegment::ModulePath,
            LogSegment::String(" @ ".to_string()),
            LogSegment::FilePath,
            LogSegment::String(":".to_string()),
            LogSegment::LineNumber,
        ];

        let result = parse(log_template);
        assert_eq!(result, Ok(expected_output));
    }

    #[test]
    fn test_parse_string_segment() {
        let result = parse_string_segment::<()>("Log: {t}");
        let (input, output) = result.unwrap();
        assert_eq!(input, "{t}");
        assert_eq!(output, LogSegment::String("Log: ".to_string()));
    }

    #[test]
    fn test_parse_empty_string_segment() {
        let result = parse_string_segment::<()>("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_timestamp_argument() {
        let result = parse_argument::<()>("{t}");
        assert_eq!(result, Ok(("", LogSegment::Timestamp)));
    }

    #[test]
    fn test_parse_invalid_argument() {
        let result = parse_argument::<()>("{foo}");
        assert_eq!(result, Result::Err(nom::Err::Error(())));
    }
}
