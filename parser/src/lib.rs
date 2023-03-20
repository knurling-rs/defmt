//! Parsing library for [`defmt`] format strings.
//!
//! This is an implementation detail of [`defmt`] and not meant to be consumed by other tools at the
//! moment so all the API is unstable.
//!
//! [`defmt`]: https://github.com/knurling-rs/defmt

#![cfg(feature = "unstable")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg(unstable)))]
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

mod types;

use std::{borrow::Cow, ops::Range, str::FromStr};

pub use crate::types::Type;

/// The kinds of error this library can return
#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum Error {
    #[error("invalid type specifier `{0:?}`")]
    InvalidTypeSpecifier(String),
    #[error("unable to parse given integer")]
    InvalidInteger(#[from] std::num::ParseIntError),
    #[error("invalid array specifier (missing length)")]
    InvalidArraySpecifierMissingLength,
    #[error("invalid array specifier (missing `]`")]
    InvalidArraySpecifierMissingBracket,
    #[error("trailing data after bitfield range")]
    TrailingDataAfterBitfieldRange,
    #[error("malformed format string (missing display hint after ':')")]
    MalformedFormatString,
    #[error("unknown display hint: {0:?}")]
    UnknownDisplayHint(String),
    #[error("unexpected content `{0:?}` in format string")]
    UnexpectedContentInFormatString(String),
    #[error("unmatched `{{` in format string")]
    UnmatchedOpenBracket,
    #[error("unmatched `}}` in format string")]
    UnmatchedCloseBracket,
    #[error("conflicting types for argument {0}: used as {1:?} and {2:?}")]
    ConflictingTypes(usize, Type, Type),
    #[error("argument {0} is not used in this format string")]
    UnusedArgument(usize),
}

/// A parameter of the form `{{0=Type:hint}}` in a format string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    /// The argument index to display at this position.
    pub index: usize,
    /// The type of the argument to display, e.g. '=u8', '=bool'.
    pub ty: Type,
    /// The display hint, e.g. ':x', ':b', ':a'.
    pub hint: Option<DisplayHint>,
}

/// Precision of ISO8601 datetime
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Millis,
    Seconds,
}

/// All display hints
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayHint {
    NoHint {
        zero_pad: usize,
    },
    /// `:x` OR `:X`
    Hexadecimal {
        alternate: bool,
        uppercase: bool,
        zero_pad: usize,
    },
    /// `:b`
    Binary {
        alternate: bool,
        zero_pad: usize,
    },
    /// `:a`
    Ascii,
    /// `:?`
    Debug,
    /// `:us`, formats integers as timestamps in microseconds
    Microseconds,
    /// `:iso8601{ms,s}`, formats integers as timestamp in ISO8601 date time format
    ISO8601(TimePrecision),
    /// `__internal_bitflags_NAME` instructs the decoder to print the flags that are set, instead of
    /// the raw value.
    Bitflags {
        name: String,
        package: String,
        disambiguator: String,
    },
    /// Display hints currently not supported / understood
    Unknown(String),
}

/// Parses the display hint (e.g. the `#x` in `{=u8:#x}`)
fn parse_display_hint(mut s: &str) -> Option<DisplayHint> {
    const BITFLAGS_HINT_START: &str = "__internal_bitflags_";

    // The `#` comes before any padding hints (I think this matches core::fmt).
    // It is ignored for types that don't have an alternate representation.
    let alternate = if matches!(s.chars().next(), Some('#')) {
        s = &s[1..]; // '#' is always 1 byte
        true
    } else {
        false
    };

    let zero_pad = if let Some(rest) = s.strip_prefix('0') {
        let (rest, columns) = parse_integer::<usize>(rest)?;
        s = rest;
        columns
    } else {
        0 // default behavior is the same as no zero-padding.
    };

    if let Some(stripped) = s.strip_prefix(BITFLAGS_HINT_START) {
        let parts = stripped.split('@').collect::<Vec<_>>();
        match *parts {
            [bitflags_name, package, disambiguator] => {
                return Some(DisplayHint::Bitflags {
                    name: bitflags_name.into(),
                    package: package.into(),
                    disambiguator: disambiguator.into(),
                });
            }
            _ => {
                return Some(DisplayHint::Unknown(s.into()));
            }
        }
    }

    Some(match s {
        "" => DisplayHint::NoHint { zero_pad },
        "us" => DisplayHint::Microseconds,
        "a" => DisplayHint::Ascii,
        "b" => DisplayHint::Binary {
            alternate,
            zero_pad,
        },
        "x" => DisplayHint::Hexadecimal {
            alternate,
            uppercase: false,
            zero_pad,
        },
        "X" => DisplayHint::Hexadecimal {
            alternate,
            uppercase: true,
            zero_pad,
        },
        "iso8601ms" => DisplayHint::ISO8601(TimePrecision::Millis),
        "iso8601s" => DisplayHint::ISO8601(TimePrecision::Seconds),
        "?" => DisplayHint::Debug,
        _ => return None,
    })
}

/// A part of a format string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fragment<'f> {
    /// A literal string (eg. `"literal "` in `"literal {:?}"`).
    Literal(Cow<'f, str>),

    /// A format parameter.
    Parameter(Parameter),
}

/// A parsed formatting parameter (contents of `{` `}` block).
///
/// # Syntax
///
/// ```notrust
/// param := '{' [ argument ] [ '=' argtype ] [ ':' format_spec ] '}'
/// argument := integer
///
/// argtype := bitfield | '?' | format-array | '[?]' | byte-array | '[u8]' | 'istr' | 'str' |
///     'bool' | 'char' | 'u8' | 'u16' | 'u32' | 'u64' | 'u128' | 'usize' | 'i8' | 'i16' | 'i32' |
///     'i64' | 'i128 | 'isize' | 'f32' | 'f64'
/// bitfield := integer '..' integer
/// format-array := '[?;' spaces integer ']'
/// byte-array := '[u8;' spaces integer ']'
/// spaces := ' '*
///
/// format_spec := [ zero_pad ] type
/// zero_pad := '0' integer
/// type := 'a' | 'b' | 'o' | 'x' | 'X' | '?' | 'us'
/// ```
#[derive(Debug, PartialEq)]
struct Param {
    index: Option<usize>,
    ty: Type,
    hint: Option<DisplayHint>,
}

/// The log level
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }
}

/// Parses an integer at the beginning of `s`.
///
/// Returns the integer and remaining text, if `s` started with an integer. Any errors parsing the
/// number (which we already know only contains digits) are silently ignored.
fn parse_integer<T: FromStr>(s: &str) -> Option<(&str, usize)> {
    let start_digits = s
        .as_bytes()
        .iter()
        .copied()
        .take_while(|b| b.is_ascii_digit())
        .count();
    let num = s[..start_digits].parse().ok()?;
    Some((&s[start_digits..], num))
}

fn parse_range(mut s: &str) -> Option<(Range<u8>, usize /* consumed */)> {
    // consume first number
    let start_digits = s
        .as_bytes()
        .iter()
        .take_while(|b| (**b as char).is_ascii_digit())
        .count();
    let start = s[..start_digits].parse().ok()?;

    // next two `char`s should be `..`
    if &s[start_digits..start_digits + 2] != ".." {
        return None;
    }
    s = &s[start_digits + 2..];

    // consume second number
    let end_digits = s
        .as_bytes()
        .iter()
        .take_while(|b| (**b as char).is_ascii_digit())
        .count();
    let end = s[..end_digits].parse().ok()?;

    // check for faulty state
    if end <= start || start >= 128 || end > 128 {
        return None;
    }

    Some((start..end, start_digits + end_digits + 2))
}

fn parse_array(mut s: &str) -> Result<usize, Error> {
    // skip spaces
    let len_pos = s
        .find(|c: char| c != ' ')
        .ok_or(Error::InvalidArraySpecifierMissingLength)?;
    s = &s[len_pos..];

    // consume length
    let after_len = s
        .find(|c: char| !c.is_ascii_digit())
        .ok_or(Error::InvalidArraySpecifierMissingBracket)?;
    let len = s[..after_len].parse::<usize>()?;
    s = &s[after_len..];

    // consume final `]`
    if s != "]" {
        return Err(Error::InvalidArraySpecifierMissingBracket);
    }

    Ok(len)
}

/// Parser mode
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParserMode {
    /// Rejects unknown display hints
    Strict,
    /// Accepts unknown display hints
    ForwardsCompatible,
}

/// Parse `Param` from `&str`
///
/// * example `input`: `0=Type:hint` (note: no curly braces)
fn parse_param(mut input: &str, mode: ParserMode) -> Result<Param, Error> {
    const TYPE_PREFIX: &str = "=";
    const HINT_PREFIX: &str = ":";

    // First, optional argument index.
    let mut index = None;
    let index_end = input
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(input.len());

    if index_end != 0 {
        index = Some(input[..index_end].parse::<usize>()?);
    }

    // Then, optional type
    let mut ty = Type::default(); // when no explicit type; use the default one
    input = &input[index_end..];

    if input.starts_with(TYPE_PREFIX) {
        // skip the prefix
        input = &input[TYPE_PREFIX.len()..];

        // type is delimited by `HINT_PREFIX` or end-of-string
        let type_end = input.find(HINT_PREFIX).unwrap_or(input.len());
        let type_fragment = &input[..type_end];

        const FORMAT_ARRAY_START: &str = "[?;";
        const U8_ARRAY_START: &str = "[u8;";

        // what comes next is the type
        ty = match type_fragment.parse() {
            Ok(ty) => ty,
            _ if input.starts_with(U8_ARRAY_START) => {
                let len = parse_array(&type_fragment[U8_ARRAY_START.len()..])?;
                Type::U8Array(len)
            }
            _ if input.starts_with(FORMAT_ARRAY_START) => {
                let len = parse_array(&type_fragment[FORMAT_ARRAY_START.len()..])?;
                Type::FormatArray(len)
            }
            _ => match parse_range(type_fragment) {
                // Check for bitfield syntax.
                Some((_, used)) if used != type_fragment.len() => {
                    return Err(Error::TrailingDataAfterBitfieldRange);
                }
                Some((range, _)) => Type::BitField(range),
                None => {
                    return Err(Error::InvalidTypeSpecifier(input.to_owned()));
                }
            },
        };

        input = &input[type_end..];
    }

    // Then, optional hint
    let mut hint = None;

    if input.starts_with(HINT_PREFIX) {
        // skip the prefix
        input = &input[HINT_PREFIX.len()..];
        if input.is_empty() {
            return Err(Error::MalformedFormatString);
        }

        hint = Some(match parse_display_hint(input) {
            Some(a) => a,
            None => match mode {
                ParserMode::Strict => {
                    return Err(Error::UnknownDisplayHint(input.to_owned()));
                }
                ParserMode::ForwardsCompatible => DisplayHint::Unknown(input.to_owned()),
            },
        });
    } else if !input.is_empty() {
        return Err(Error::UnexpectedContentInFormatString(input.to_owned()));
    }

    Ok(Param { index, ty, hint })
}

fn push_literal<'f>(frag: &mut Vec<Fragment<'f>>, unescaped_literal: &'f str) -> Result<(), Error> {
    // Replace `{{` with `{` and `}}` with `}`. Single braces are errors.

    // Scan for single braces first. The rest is trivial.
    let mut last_open = false;
    let mut last_close = false;
    for c in unescaped_literal.chars() {
        match c {
            '{' => last_open = !last_open,
            '}' => last_close = !last_close,
            _ => {
                if last_open {
                    return Err(Error::UnmatchedOpenBracket);
                }
                if last_close {
                    return Err(Error::UnmatchedCloseBracket);
                }
            }
        }
    }

    // Handle trailing unescaped `{` or `}`.
    if last_open {
        return Err(Error::UnmatchedOpenBracket);
    }
    if last_close {
        return Err(Error::UnmatchedCloseBracket);
    }

    // FIXME: This always allocates a `String`, so the `Cow` is useless.
    let literal = unescaped_literal.replace("{{", "{").replace("}}", "}");
    frag.push(Fragment::Literal(literal.into()));
    Ok(())
}

/// Returns `Some(smallest_bit_index, largest_bit_index)` contained in `params` if
/// `params` contains any bitfields. Otherwise `None`.
pub fn get_max_bitfield_range<'a, I>(params: I) -> Option<(u8, u8)>
where
    I: Iterator<Item = &'a Parameter> + Clone,
{
    let largest_bit_index = params
        .clone()
        .map(|param| match &param.ty {
            Type::BitField(range) => range.end,
            _ => unreachable!(),
        })
        .max();

    let smallest_bit_index = params
        .map(|param| match &param.ty {
            Type::BitField(range) => range.start,
            _ => unreachable!(),
        })
        .min();

    match (smallest_bit_index, largest_bit_index) {
        (Some(smallest), Some(largest)) => Some((smallest, largest)),
        (None, None) => None,
        _ => unreachable!(),
    }
}

pub fn parse(format_string: &str, mode: ParserMode) -> Result<Vec<Fragment<'_>>, Error> {
    let mut fragments = Vec::new();

    // Index after the `}` of the last format specifier.
    let mut end_pos = 0;

    // Next argument index assigned to a parameter without an explicit one.
    let mut next_arg_index = 0;

    let mut chars = format_string.char_indices();
    while let Some((brace_pos, ch)) = chars.next() {
        if ch != '{' {
            // Part of a literal fragment.
            continue;
        }

        // Peek at the next char.
        if chars.as_str().starts_with('{') {
            // Escaped `{{`, also part of a literal fragment.
            chars.next(); // Move after both `{`s.
            continue;
        }

        if brace_pos > end_pos {
            // There's a literal fragment with at least 1 character before this parameter fragment.
            let unescaped_literal = &format_string[end_pos..brace_pos];
            push_literal(&mut fragments, unescaped_literal)?;
        }

        // Else, this is a format specifier. It ends at the next `}`.
        let len = chars
            .as_str()
            .find('}')
            .ok_or(Error::UnmatchedOpenBracket)?;
        end_pos = brace_pos + 1 + len + 1;

        // Parse the contents inside the braces.
        let param_str = &format_string[brace_pos + 1..][..len];
        let param = parse_param(param_str, mode)?;
        fragments.push(Fragment::Parameter(Parameter {
            index: param.index.unwrap_or_else(|| {
                // If there is no explicit index, assign the next one.
                let idx = next_arg_index;
                next_arg_index += 1;
                idx
            }),
            ty: param.ty,
            hint: param.hint,
        }));
    }

    // Trailing literal.
    if end_pos != format_string.len() {
        push_literal(&mut fragments, &format_string[end_pos..])?;
    }

    // Check for argument type conflicts.
    let mut args = Vec::new();
    for frag in &fragments {
        if let Fragment::Parameter(Parameter { index, ty, .. }) = frag {
            if args.len() <= *index {
                args.resize(*index + 1, None);
            }

            match &mut args[*index] {
                none @ None => {
                    *none = Some(ty.clone());
                }
                Some(other_ty) => match (other_ty, ty) {
                    // FIXME: Bitfield range shouldn't be part of the type.
                    (Type::BitField(_), Type::BitField(_)) => {}
                    (a, b) if a != b => {
                        return Err(Error::ConflictingTypes(*index, a.clone(), ty.clone()));
                    }
                    _ => {}
                },
            }
        }
    }

    // Check that argument indices are dense (all arguments must be used).
    for (index, arg) in args.iter().enumerate() {
        if arg.is_none() {
            return Err(Error::UnusedArgument(index));
        }
    }

    Ok(fragments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    // no `Param` field present - 1 case
    #[case("", None, Type::Format, None)]
    // only one `Param` field present - 3 cases
    #[case("=u8", None, Type::U8, None)]
    #[case(":a", None, Type::Format, Some(DisplayHint::Ascii))]
    #[case("1", Some(1), Type::Format, None)]
    // two `Param` fields present - 3 cases
    #[case("=u8:x", None, Type::U8, Some(DisplayHint::Hexadecimal {alternate: false, uppercase: false, zero_pad: 0}))]
    #[case("0=u8", Some(0), Type::U8, None)]
    #[case("0:a", Some(0), Type::Format, Some(DisplayHint::Ascii))]
    // all `Param` fields present - 1 case
    #[case("1=u8:b", Some(1), Type::U8, Some(DisplayHint::Binary { alternate: false, zero_pad: 0}))]
    fn all_parse_param_cases(
        #[case] input: &str,
        #[case] index: Option<usize>,
        #[case] ty: Type,
        #[case] hint: Option<DisplayHint>,
    ) {
        assert_eq!(
            parse_param(input, ParserMode::Strict),
            Ok(Param { index, ty, hint })
        );
    }

    #[rstest]
    #[case(":a", DisplayHint::Ascii)]
    #[case(":b", DisplayHint::Binary { alternate: false, zero_pad: 0 })]
    #[case(":#b", DisplayHint::Binary { alternate: true, zero_pad: 0 })]
    #[case(":x", DisplayHint::Hexadecimal { alternate: false, uppercase: false, zero_pad: 0 })]
    #[case(":#x", DisplayHint::Hexadecimal { alternate: true, uppercase: false, zero_pad: 0 })]
    #[case(":X", DisplayHint::Hexadecimal { alternate: false, uppercase: true, zero_pad: 0 })]
    #[case(":#X", DisplayHint::Hexadecimal { alternate: true, uppercase: true, zero_pad: 0 })]
    #[case(":iso8601ms", DisplayHint::ISO8601(TimePrecision::Millis))]
    #[case(":iso8601s", DisplayHint::ISO8601(TimePrecision::Seconds))]
    #[case(":?", DisplayHint::Debug)]
    #[case(":02", DisplayHint::NoHint { zero_pad: 2 })]
    fn all_display_hints(#[case] input: &str, #[case] hint: DisplayHint) {
        assert_eq!(
            parse_param(input, ParserMode::Strict),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(hint),
            })
        );
    }

    #[test]
    // separate test, because of `ParserMode::ForwardsCompatible`
    fn display_hint_unknown() {
        assert_eq!(
            parse_param(":unknown", ParserMode::ForwardsCompatible),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Unknown("unknown".to_string())),
            })
        );
    }

    #[rstest]
    #[case("=i8", Type::I8)]
    #[case("=i16", Type::I16)]
    #[case("=i32", Type::I32)]
    #[case("=i64", Type::I64)]
    #[case("=i128", Type::I128)]
    #[case("=isize", Type::Isize)]
    #[case("=u8", Type::U8)]
    #[case("=u16", Type::U16)]
    #[case("=u32", Type::U32)]
    #[case("=u64", Type::U64)]
    #[case("=u128", Type::U128)]
    #[case("=usize", Type::Usize)]
    #[case("=f32", Type::F32)]
    #[case("=f64", Type::F64)]
    #[case("=bool", Type::Bool)]
    #[case("=?", Type::Format)]
    #[case("=str", Type::Str)]
    #[case("=[u8]", Type::U8Slice)]
    fn all_types(#[case] input: &str, #[case] ty: Type) {
        assert_eq!(
            parse_param(input, ParserMode::Strict),
            Ok(Param {
                index: None,
                ty,
                hint: None,
            })
        );
    }

    #[rstest]
    #[case::implicit("{=u8}{=u16}", [(0, Type::U8), (1, Type::U16)])]
    #[case::single_parameter_formatted_twice("{=u8}{0=u8}", [(0, Type::U8), (0, Type::U8)])]
    #[case::explicit_index("{=u8}{1=u16}", [(0, Type::U8), (1, Type::U16)])]
    #[case::reversed_order("{1=u8}{0=u16}", [(1, Type::U8), (0, Type::U16)])]
    fn index(#[case] input: &str, #[case] params: [(usize, Type); 2]) {
        assert_eq!(
            parse(input, ParserMode::Strict),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: params[0].0,
                    ty: params[0].1.clone(),
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: params[1].0,
                    ty: params[1].1.clone(),
                    hint: None,
                }),
            ])
        );
    }

    #[rstest]
    #[case::different_types_for_same_index("{0=u8}{0=u16}")]
    #[case::same_thing_except_bool_is_autoassigned_index_0("Hello {1=u16} {0=u8} {=bool}")]
    #[case::omitted_index_0("{1=u8}")]
    #[case::index_1_is_missing("{2=u8}{=u16}")]
    #[case::index_0_is_missing("{2=u8}{1=u16}")]
    fn index_err(#[case] input: &str) {
        assert!(parse(input, ParserMode::Strict).is_err());
    }

    #[rstest]
    #[case("{=0..4}", 0..4)]
    #[case::just_inside_128bit_range_1("{=0..128}", 0..128)]
    #[case::just_inside_128bit_range_2("{=127..128}", 127..128)]
    fn range(#[case] input: &str, #[case] bit_field: Range<u8>) {
        assert_eq!(
            parse(input, ParserMode::Strict),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::BitField(bit_field),
                hint: None,
            })])
        );
    }

    #[test]
    fn multiple_ranges() {
        assert_eq!(
            parse("{0=30..31}{1=0..4}{1=2..6}", ParserMode::Strict),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::BitField(30..31),
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::BitField(0..4),
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::BitField(2..6),
                    hint: None,
                }),
            ])
        );
    }

    #[rstest]
    #[case::empty_range("{=0..0}")]
    #[case::start_gt_end("{=1..0}")]
    #[case::out_of_128bit_range_1("{=0..129}")]
    #[case::out_of_128bit_range_2("{=128..128}")]
    #[case::missing_parts_1("{=0..4")]
    #[case::missing_parts_2("{=0..}")]
    #[case::missing_parts_3("{=..4}")]
    #[case::missing_parts_4("{=0.4}")]
    #[case::missing_parts_5("{=0...4}")]
    fn range_err(#[case] input: &str) {
        assert!(parse(input, ParserMode::Strict).is_err());
    }

    #[test]
    fn arrays() {
        assert_eq!(
            parse("{=[u8; 0]}", ParserMode::Strict),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(0),
                hint: None,
            })])
        );

        // Space is optional.
        assert_eq!(
            parse("{=[u8;42]}", ParserMode::Strict),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(42),
                hint: None,
            })])
        );

        // Multiple spaces are ok.
        assert_eq!(
            parse("{=[u8;    257]}", ParserMode::Strict),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(257),
                hint: None,
            })])
        );

        // No tabs or other whitespace.
        assert!(parse("{=[u8; \t 3]}", ParserMode::Strict).is_err());
        assert!(parse("{=[u8; \n 3]}", ParserMode::Strict).is_err());
        // Too large.
        assert!(parse("{=[u8; 9999999999999999999999999]}", ParserMode::Strict).is_err());
    }

    #[rstest]
    #[case("{=dunno}", Error::InvalidTypeSpecifier("dunno".to_string()))]
    #[case("{dunno}", Error::UnexpectedContentInFormatString("dunno".to_string()))]
    #[case("{=u8;x}", Error::InvalidTypeSpecifier("u8;x".to_string()))]
    #[case("{dunno=u8:x}", Error::UnexpectedContentInFormatString("dunno=u8:x".to_string()))]
    #[case("{0dunno}", Error::UnexpectedContentInFormatString("dunno".to_string()))]
    #[case("{:}", Error::MalformedFormatString)]
    fn error_msg(#[case] input: &str, #[case] err: Error) {
        assert_eq!(parse(input, ParserMode::Strict), Err(err));
    }

    #[rstest]
    #[case("}string")]
    #[case("{string")]
    #[case("}")]
    #[case("{")]
    fn stray_braces(#[case] input: &str) {
        assert!(parse(input, ParserMode::Strict).is_err());
    }

    #[rstest]
    #[case("}}", "}")]
    #[case("{{", "{")]
    #[case("literal{{literal", "literal{literal")]
    #[case("literal}}literal", "literal}literal")]
    #[case("{{}}", "{}")]
    #[case("}}{{", "}{")]
    fn escaped_braces(#[case] input: &str, #[case] literal: &str) {
        assert_eq!(
            parse(input, ParserMode::Strict),
            Ok(vec![Fragment::Literal(literal.into())])
        );
    }
}
