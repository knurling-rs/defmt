use nom::{
    branch::alt,
    bytes::complete::{take, take_till1},
    character::complete::char,
    combinator::{map, map_res},
    multi::many0,
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub(super) enum LogSegment {
    FileName,
    FilePath,
    LineNumber,
    Log,
    LogLevel,
    ModulePath,
    String(String),
    Timestamp,
}

fn parse_argument(input: &str) -> IResult<&str, LogSegment, ()> {
    let parse_enclosed = delimited(char('{'), take(1u32), char('}'));
    let mut parse_type = map_res(parse_enclosed, move |s| match s {
        "f" => Ok(LogSegment::FileName),
        "F" => Ok(LogSegment::FilePath),
        "l" => Ok(LogSegment::LineNumber),
        "s" => Ok(LogSegment::Log),
        "L" => Ok(LogSegment::LogLevel),
        "m" => Ok(LogSegment::ModulePath),
        "t" => Ok(LogSegment::Timestamp),
        _ => Err(()),
    });

    parse_type.parse(input)
}

fn parse_string_segment(input: &str) -> IResult<&str, LogSegment, ()> {
    map(take_till1(|c| c == '{'), |s: &str| {
        LogSegment::String(s.to_string())
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
        let result = parse_string_segment("Log: {t}");
        let (input, output) = result.unwrap();
        assert_eq!(input, "{t}");
        assert_eq!(output, LogSegment::String("Log: ".to_string()));
    }

    #[test]
    fn test_parse_empty_string_segment() {
        let result = parse_string_segment("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_timestamp_argument() {
        let result = parse_argument("{t}");
        assert_eq!(result, Ok(("", LogSegment::Timestamp)));
    }

    #[test]
    fn test_parse_invalid_argument() {
        let result = parse_argument("{foo}");
        assert_eq!(result, Result::Err(nom::Err::Error(())));
    }
}
