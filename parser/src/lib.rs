//! Parsing library for [`defmt`](https://github.com/knurling-rs/defmt) format strings.
//!
//! This is an implementation detail of [`defmt`] and not meant to be consumed by other tools at the
//! moment so all the API is unstable.

#![cfg(feature = "unstable")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg(unstable)))]

use std::borrow::Cow;
use std::ops::Range;

/// A parameter of the form `{{0=Type:hint}}` in a format string.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// The argument index to display at this position.
    pub index: usize,
    /// The type of the argument to display, e.g. '=u8', '=bool'.
    pub ty: Type,
    /// The display hint, e.g. ':x', ':b', ':a'.
    pub hint: Option<DisplayHint>,
}

/// All display hints
#[derive(Clone, Debug, PartialEq)]
pub enum DisplayHint {
    /// ":x" OR ":X"
    Hexadecimal { is_uppercase: bool },
    /// ":b"
    Binary,
    /// ":a"
    Ascii,
    /// ":?"
    Debug,
    /// Display hints currently not supported / understood
    Unknown(String),
}

/// A part of a format string.
#[derive(Clone, Debug, PartialEq)]
pub enum Fragment<'f> {
    /// A literal string (eg. `"literal "` in `"literal {:?}"`).
    Literal(Cow<'f, str>),

    /// A format parameter.
    Parameter(Parameter),
}

#[derive(Debug, PartialEq)]
struct Param {
    index: Option<usize>,
    ty: Type,
    hint: Option<DisplayHint>,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    BitField(Range<u8>),
    Bool,
    Format,             // "{=?}" OR "{}"
    FormatSlice,        // "{=[?]}"
    FormatArray(usize), // FIXME: This `usize` is not the target's `usize`; use `u64` instead?
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    /// String slice (i.e. passed directly; not as interned string indices).
    Str,
    /// Interned string index.
    IStr,
    U8,
    U16,
    U24,
    U32,
    U64,
    U128,
    Usize,
    /// Byte slice `{:[u8]}`.
    U8Slice,
    U8Array(usize), // FIXME: This `usize` is not the target's `usize`; use `u64` instead?
    F32,
    /// A single Unicode character
    Char,
}

// when not specified in the format string, this type is assumed
impl Default for Type {
    fn default() -> Self {
        Type::Format
    }
}

fn is_digit(c: Option<char>) -> bool {
    matches!(c.unwrap_or('\0'), '0'..='9')
}

fn parse_range(mut s: &str) -> Option<(Range<u8>, usize /* consumed */)> {
    let start_digits = s
        .as_bytes()
        .iter()
        .take_while(|b| is_digit(Some(**b as char)))
        .count();
    let start = s[..start_digits].parse().ok()?;
    if &s[start_digits..start_digits + 2] != ".." {
        return None;
    }
    s = &s[start_digits + 2..];
    let end_digits = s
        .as_bytes()
        .iter()
        .take_while(|b| is_digit(Some(**b as char)))
        .count();
    let end = s[..end_digits].parse().ok()?;

    if end <= start {
        return None;
    }

    if start >= 32 || end >= 32 {
        return None;
    }

    Some((start..end, start_digits + end_digits + 2))
}

fn parse_array(mut s: &str) -> Result<usize, Cow<'static, str>> {
    // Skip spaces.
    let len_pos = s
        .find(|c: char| c != ' ')
        .ok_or("invalid array specifier (missing length)")?;
    s = &s[len_pos..];
    let after_len = s
        .find(|c: char| !c.is_digit(10))
        .ok_or("invalid array specifier (missing `]`)")?;
    let len = s[..after_len].parse::<usize>().map_err(|e| e.to_string())?;
    s = &s[after_len..];
    if s != "]" {
        return Err("invalid array specifier (missing `]`)".into());
    }

    Ok(len)
}

/// Parser mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParserMode {
    /// Rejects unknown display hints
    Strict,

    /// Accepts unknown display hints
    ForwardsCompatible,
}

// example `input`: "0=Type:hint" (note: no curly braces)
fn parse_param(mut input: &str, mode: ParserMode) -> Result<Param, Cow<'static, str>> {
    const TYPE_PREFIX: &str = "=";
    const HINT_PREFIX: &str = ":";

    // First, optional argument index.
    let mut index = None;
    let index_end = input.find(|c: char| !c.is_digit(10)).unwrap_or(input.len());

    if index_end != 0 {
        index = Some(
            input[..index_end]
                .parse::<usize>()
                .map_err(|e| e.to_string())?,
        );
    }

    // Then, optional type
    let mut ty = Type::default(); // when no explicit type; use the default one
    input = &input[index_end..];

    if input.starts_with(TYPE_PREFIX) {
        // skip the prefix
        input = &input[TYPE_PREFIX.len()..];

        // type is delimited by `HINT_PREFIX` or end-of-string
        let type_end = input.find(HINT_PREFIX).unwrap_or(input.len());

        static FORMAT_ARRAY_START: &str = "[?;";
        static U8_ARRAY_START: &str = "[u8;";

        // what comes next is the type
        ty = match &input[..type_end] {
            "u8" => Type::U8,
            "u16" => Type::U16,
            "u24" => Type::U24,
            "u32" => Type::U32,
            "u64" => Type::U64,
            "u128" => Type::U128,
            "usize" => Type::Usize,
            "i8" => Type::I8,
            "i16" => Type::I16,
            "i32" => Type::I32,
            "i64" => Type::I64,
            "i128" => Type::I128,
            "isize" => Type::Isize,
            "f32" => Type::F32,
            "bool" => Type::Bool,
            "str" => Type::Str,
            "istr" => Type::IStr,
            "[u8]" => Type::U8Slice,
            "?" => Type::Format,
            "[?]" => Type::FormatSlice,
            "char" => Type::Char,
            _ if input.starts_with(U8_ARRAY_START) => {
                let len = parse_array(&input[U8_ARRAY_START.len()..type_end])?;
                Type::U8Array(len)
            }
            _ if input.starts_with(FORMAT_ARRAY_START) => {
                let len = parse_array(&input[FORMAT_ARRAY_START.len()..type_end])?;
                Type::FormatArray(len)
            }
            _ => {
                // Check for bitfield syntax.
                match parse_range(input) {
                    Some((range, used)) => {
                        if used != input.len() {
                            return Err("trailing data after bitfield range".into());
                        }

                        Type::BitField(range)
                    }
                    None => {
                        return Err(format!(
                            "malformed format string (invalid type specifier `{}`)",
                            input
                        )
                        .into());
                    }
                }
            }
        };

        input = &input[type_end..];
    }

    // Then, optional hint
    let mut hint = None;

    if input.starts_with(HINT_PREFIX) {
        // skip the prefix
        input = &input[HINT_PREFIX.len()..];

        hint = Some(match input {
            "a" => DisplayHint::Ascii,
            "b" => DisplayHint::Binary,
            "x" => DisplayHint::Hexadecimal {
                is_uppercase: false,
            },
            "X" => DisplayHint::Hexadecimal { is_uppercase: true },
            "?" => DisplayHint::Debug,
            _ => match mode {
                ParserMode::Strict => {
                    return Err(format!("unknown display hint: {:?}", input).into());
                }
                ParserMode::ForwardsCompatible => DisplayHint::Unknown(input.to_owned()),
            },
        });
    } else if !input.is_empty() {
        return Err(format!("unexpected content {:?} in format string", input).into());
    }

    Ok(Param { index, ty, hint })
}

fn push_literal<'f>(
    frag: &mut Vec<Fragment<'f>>,
    unescaped_literal: &'f str,
) -> Result<(), Cow<'static, str>> {
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
                    return Err("unmatched `{` in format string".into());
                }
                if last_close {
                    return Err("unmatched `}` in format string".into());
                }
            }
        }
    }

    // Handle trailing unescaped `{` or `}`.
    if last_open {
        return Err("unmatched `{` in format string".into());
    }
    if last_close {
        return Err("unmatched `}` in format string".into());
    }

    // FIXME: This always allocates a `String`, so the `Cow` is useless.
    let literal = unescaped_literal.replace("{{", "{").replace("}}", "}");
    frag.push(Fragment::Literal(literal.into()));
    Ok(())
}

/// returns Some(smallest_bit_index, largest_bit_index) contained in `params` if
///         `params` contains any bitfields.
///         None otherwise
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

pub fn parse<'f>(
    format_string: &'f str,
    mode: ParserMode,
) -> Result<Vec<Fragment<'f>>, Cow<'static, str>> {
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
            .ok_or("missing `}` in format string")?;
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
                Some(other_ty) => {
                    // FIXME: Bitfield range shouldn't be part of the type.
                    match (&*other_ty, ty) {
                        (Type::BitField(_), Type::BitField(_)) => {}
                        (a, b) if a != b => {
                            return Err(format!(
                                "conflicting types for argument {}: used as {:?} and {:?}",
                                index, other_ty, ty
                            )
                            .into());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Check that argument indices are dense (all arguments must be used).
    for (index, arg) in args.iter().enumerate() {
        if arg.is_none() {
            return Err(format!("argument {} is not used in this format string", index).into());
        }
    }

    Ok(fragments)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_parse_param_cases() {
        // no `Param` field present - 1 case
        assert_eq!(
            parse_param(""),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: None,
            })
        );

        // only one `Param` field present - 3 cases
        assert_eq!(
            parse_param("=u8"),
            Ok(Param {
                index: None,
                ty: Type::U8,
                hint: None,
            })
        );

        assert_eq!(
            parse_param(":a"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Ascii),
            })
        );

        assert_eq!(
            parse_param("1"),
            Ok(Param {
                index: Some(1),
                ty: Type::Format,
                hint: None,
            })
        );

        // two `Param` fields present - 3 cases
        assert_eq!(
            parse_param("=u8:x"),
            Ok(Param {
                index: None,
                ty: Type::U8,
                hint: Some(DisplayHint::Hexadecimal {
                    is_uppercase: false
                }),
            })
        );

        assert_eq!(
            parse_param("0=u8"),
            Ok(Param {
                index: Some(0),
                ty: Type::U8,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("0:a"),
            Ok(Param {
                index: Some(0),
                ty: Type::Format,
                hint: Some(DisplayHint::Ascii),
            })
        );

        // all `Param` fields present - 1 case
        assert_eq!(
            parse_param("1=u8:b"),
            Ok(Param {
                index: Some(1),
                ty: Type::U8,
                hint: Some(DisplayHint::Binary),
            })
        );
    }

    #[test]
    fn all_display_hints() {
        assert_eq!(
            parse_param(":a"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Ascii),
            })
        );

        assert_eq!(
            parse_param(":b"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Binary),
            })
        );

        assert_eq!(
            parse_param(":x"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Hexadecimal {
                    is_uppercase: false
                }),
            })
        );

        assert_eq!(
            parse_param(":X"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Hexadecimal { is_uppercase: true }),
            })
        );

        assert_eq!(
            parse_param(":?"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Debug),
            })
        );

        assert_eq!(
            parse_param(":unknown"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: Some(DisplayHint::Unknown("unknown".to_string())),
            })
        );
    }

    #[test]
    fn all_types() {
        assert_eq!(
            parse_param("=bool"),
            Ok(Param {
                index: None,
                ty: Type::Bool,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=?"),
            Ok(Param {
                index: None,
                ty: Type::Format,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=i16"),
            Ok(Param {
                index: None,
                ty: Type::I16,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=i32"),
            Ok(Param {
                index: None,
                ty: Type::I32,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=i64"),
            Ok(Param {
                index: None,
                ty: Type::I64,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=i128"),
            Ok(Param {
                index: None,
                ty: Type::I128,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=i8"),
            Ok(Param {
                index: None,
                ty: Type::I8,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=str"),
            Ok(Param {
                index: None,
                ty: Type::Str,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u16"),
            Ok(Param {
                index: None,
                ty: Type::U16,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u24"),
            Ok(Param {
                index: None,
                ty: Type::U24,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u32"),
            Ok(Param {
                index: None,
                ty: Type::U32,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u64"),
            Ok(Param {
                index: None,
                ty: Type::U64,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u128"),
            Ok(Param {
                index: None,
                ty: Type::U128,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=f32"),
            Ok(Param {
                index: None,
                ty: Type::F32,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=u8"),
            Ok(Param {
                index: None,
                ty: Type::U8,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=[u8]"),
            Ok(Param {
                index: None,
                ty: Type::U8Slice,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=usize"),
            Ok(Param {
                index: None,
                ty: Type::Usize,
                hint: None,
            })
        );

        assert_eq!(
            parse_param("=isize"),
            Ok(Param {
                index: None,
                ty: Type::Isize,
                hint: None,
            })
        );
    }

    #[test]
    fn index() {
        // implicit
        assert_eq!(
            parse("{=u8}{=u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U16,
                    hint: None,
                }),
            ])
        );

        // single parameter formatted twice
        assert_eq!(
            parse("{=u8}{0=u8}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                    hint: None,
                }),
            ])
        );

        // explicit index
        assert_eq!(
            parse("{=u8}{1=u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U16,
                    hint: None,
                }),
            ])
        );

        // reversed order
        assert_eq!(
            parse("{1=u8}{0=u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U8,
                    hint: None,
                }),
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U16,
                    hint: None,
                }),
            ])
        );

        // two different types for the same index
        assert!(parse("{0=u8}{0=u16}").is_err());
        // same thing, except `{:bool}` is auto-assigned index 0
        assert!(parse("Hello {1=u16} {0=u8} {=bool}").is_err());

        // omitted index 0
        assert!(parse("{1=u8}").is_err());

        // index 1 is missing
        assert!(parse("{2=u8}{=u16}").is_err());

        // index 0 is missing
        assert!(parse("{2=u8}{1=u16}").is_err());
    }

    #[test]
    fn range() {
        assert_eq!(
            parse("{=0..4}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::BitField(0..4),
                hint: None,
            })])
        );

        assert_eq!(
            parse("{0=30..31}{1=0..4}{1=2..6}"),
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

        // empty range
        assert!(parse("{=0..0}").is_err());
        // start > end
        assert!(parse("{=1..0}").is_err());
        // out of 32-bit range
        assert!(parse("{=0..32}").is_err());
        // just inside 32-bit range
        assert!(parse("{=0..31}").is_ok());

        // missing parts
        assert!(parse("{=0..4").is_err());
        assert!(parse("{=0..}").is_err());
        assert!(parse("{=..4}").is_err());
        assert!(parse("{=0.4}").is_err());
        assert!(parse("{=0...4}").is_err());
    }

    #[test]
    fn arrays() {
        assert_eq!(
            parse("{=[u8; 0]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(0),
                hint: None,
            })])
        );

        // Space is optional.
        assert_eq!(
            parse("{=[u8;42]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(42),
                hint: None,
            })])
        );

        // Multiple spaces are ok.
        assert_eq!(
            parse("{=[u8;    257]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(257),
                hint: None,
            })])
        );

        // No tabs or other whitespace.
        assert!(parse("{=[u8; \t 3]}").is_err());
        assert!(parse("{=[u8; \n 3]}").is_err());
        // Too large.
        assert!(parse("{=[u8; 9999999999999999999999999]}").is_err());
    }

    #[test]
    fn error_msg() {
        assert_eq!(
            parse("{=dunno}"),
            Err("malformed format string (invalid type specifier `dunno`)".into())
        );

        assert_eq!(
            parse("{dunno}"),
            Err("unexpected content \"dunno\" in format string".into())
        );

        assert_eq!(
            parse("{=u8;x}"),
            Err("malformed format string (invalid type specifier `u8;x`)".into())
        );

        assert_eq!(
            parse("{dunno=u8:x}"),
            Err("unexpected content \"dunno=u8:x\" in format string".into())
        );

        assert_eq!(
            parse("{0dunno}"),
            Err("unexpected content \"dunno\" in format string".into())
        );
    }

    #[test]
    fn brace_escape() {
        // Stray braces.
        assert!(parse("}string").is_err());
        assert!(parse("{string").is_err());
        assert!(parse("}").is_err());
        assert!(parse("{").is_err());

        // Escaped braces.
        assert_eq!(parse("}}"), Ok(vec![Fragment::Literal("}".into())]));
        assert_eq!(parse("{{"), Ok(vec![Fragment::Literal("{".into())]));
        assert_eq!(
            parse("literal{{literal"),
            Ok(vec![Fragment::Literal("literal{literal".into())])
        );
        assert_eq!(
            parse("literal}}literal"),
            Ok(vec![Fragment::Literal("literal}literal".into())])
        );
        assert_eq!(parse("{{}}"), Ok(vec![Fragment::Literal("{}".into())]));
        assert_eq!(parse("}}{{"), Ok(vec![Fragment::Literal("}{".into())]));
    }
}
