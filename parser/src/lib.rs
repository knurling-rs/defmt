//! Parsing library for [`defmt`](https://github.com/knurling-rs/defmt) format strings.
//!
//! This is an implementation detail of [`defmt`] and not meant to be consumed by other tools at the
//! moment so all the API is unstable.

#![cfg(feature = "unstable")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg(unstable)))]

use std::borrow::Cow;
use std::ops::Range;

/// A `{{:parameter}}` in a format string.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    /// The argument index to display at this position.
    pub index: usize,
    /// The type of the argument to display.
    pub ty: Type,
}

/// A part of a format string.
#[derive(Clone, Debug, PartialEq)]
pub enum Fragment<'f> {
    /// A literal string (eg. `"literal "` in `"literal {:?}"`).
    Literal(Cow<'f, str>),

    /// A format parameter.
    Parameter(Parameter),
}

struct Param {
    index: Option<usize>,
    ty: Type,
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
    Format,             // "{:?}"
    FormatSlice,        // "{:[?]}"
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
    Usize,
    /// Byte slice `{:[u8]}`.
    U8Slice,
    U8Array(usize), // FIXME: This `usize` is not the target's `usize`; use `u64` instead?
    F32,
}

fn is_digit(c: Option<char>) -> bool {
    match c.unwrap_or('\0') {
        '0'..='9' => true,
        _ => false,
    }
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

fn parse_param(mut s: &str) -> Result<Param, Cow<'static, str>> {
    // First, optional argument index.
    // Then, mandatory `:`.
    let mut index = None;
    let colon_pos = s
        .find(|c: char| !c.is_digit(10))
        .ok_or("malformed format string (missing `:`)")?;

    if colon_pos != 0 {
        index = Some(s[..colon_pos].parse::<usize>().map_err(|e| e.to_string())?);
    }

    if !s[colon_pos..].starts_with(':') {
        return Err("malformed format string (missing `:`)".into());
    }

    // Then, type specifier.
    s = &s[colon_pos + 1..];

    static FORMAT_ARRAY_START: &str = "[?;";
    static U8_ARRAY_START: &str = "[u8;";
    let ty = match s {
        "u8" => Type::U8,
        "u16" => Type::U16,
        "u24" => Type::U24,
        "u32" => Type::U32,
        "u64" => Type::U64,
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
        _ if s.starts_with(U8_ARRAY_START) => {
            s = &s[U8_ARRAY_START.len()..];
            let len = parse_array(s)?;
            Type::U8Array(len)
        }
        _ if s.starts_with(FORMAT_ARRAY_START) => {
            s = &s[FORMAT_ARRAY_START.len()..];
            let len = parse_array(s)?;
            Type::FormatArray(len)
        }
        _ => {
            // Check for bitfield syntax.
            match parse_range(s) {
                Some((range, used)) => {
                    if used != s.len() {
                        return Err("trailing data after bitfield range".into());
                    }

                    Type::BitField(range)
                }
                None => {
                    return Err(format!(
                        "malformed format string (invalid type specifier `{}`)",
                        s
                    )
                    .into());
                }
            }
        }
    };

    Ok(Param { index, ty })
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

pub fn parse<'f>(format_string: &'f str) -> Result<Vec<Fragment<'f>>, Cow<'static, str>> {
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
        if chars.as_str().chars().next() == Some('{') {
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
        let param = parse_param(param_str)?;
        fragments.push(Fragment::Parameter(Parameter {
            index: param.index.unwrap_or_else(|| {
                // If there is no explicit index, assign the next one.
                let idx = next_arg_index;
                next_arg_index += 1;
                idx
            }),
            ty: param.ty,
        }));
    }

    // Trailing literal.
    if end_pos != format_string.len() {
        push_literal(&mut fragments, &format_string[end_pos..])?;
    }

    // Check for argument type conflicts.
    let mut args = Vec::new();
    for frag in &fragments {
        match frag {
            Fragment::Parameter(Parameter { index, ty }) => {
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
            _ => {}
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
    fn ty() {
        assert_eq!(
            parse("{:bool}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::Bool,
            })])
        );

        assert_eq!(
            parse("{:?}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::Format,
            })])
        );

        assert_eq!(
            parse("{:i16}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::I16,
            })])
        );

        assert_eq!(
            parse("{:i32}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::I32,
            })])
        );

        assert_eq!(
            parse("{:i64}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::I64,
            })])
        );

        assert_eq!(
            parse("{:i128}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::I128,
            })])
        );

        assert_eq!(
            parse("{:i8}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::I8,
            })])
        );

        assert_eq!(
            parse("{:str}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::Str,
            })])
        );

        assert_eq!(
            super::parse("{:u16}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U16,
            })])
        );

        assert_eq!(
            parse("{:u24}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U24,
            })])
        );

        assert_eq!(
            parse("{:u32}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U32,
            })])
        );

        assert_eq!(
            parse("{:u64}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U64,
            })])
        );

        assert_eq!(
            parse("{:f32}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::F32,
            })])
        );

        assert_eq!(
            parse("{:u8}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8,
            })])
        );

        assert_eq!(
            parse("{:[u8]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Slice,
            })])
        );

        assert_eq!(
            parse("{:usize}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::Usize,
            })])
        );

        assert_eq!(
            parse("{:isize}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::Isize,
            })])
        );
    }

    #[test]
    fn index() {
        // implicit
        assert_eq!(
            parse("{:u8}{:u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U16,
                }),
            ])
        );

        // single parameter formatted twice
        assert_eq!(
            parse("{:u8}{0:u8}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                }),
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                }),
            ])
        );

        // explicit index
        assert_eq!(
            parse("{:u8}{1:u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U8,
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U16,
                }),
            ])
        );

        // reversed order
        assert_eq!(
            parse("{1:u8}{0:u16}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::U8,
                }),
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::U16,
                }),
            ])
        );

        // two different types for the same index
        assert!(parse("{0:u8}{0:u16}").is_err());
        // same thing, except `{:bool}` is auto-assigned index 0
        assert!(parse("Hello {1:u16} {0:u8} {:bool}").is_err());

        // omitted index 0
        assert!(parse("{1:u8}").is_err());

        // index 1 is missing
        assert!(parse("{2:u8}{:u16}").is_err());

        // index 0 is missing
        assert!(parse("{2:u8}{1:u16}").is_err());
    }

    #[test]
    fn range() {
        assert_eq!(
            parse("{:0..4}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::BitField(0..4),
            })])
        );

        assert_eq!(
            parse("{0:30..31}{1:0..4}{1:2..6}"),
            Ok(vec![
                Fragment::Parameter(Parameter {
                    index: 0,
                    ty: Type::BitField(30..31),
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::BitField(0..4),
                }),
                Fragment::Parameter(Parameter {
                    index: 1,
                    ty: Type::BitField(2..6),
                }),
            ])
        );

        // empty range
        assert!(parse("{:0..0}").is_err());
        // start > end
        assert!(parse("{:1..0}").is_err());
        // out of 32-bit range
        assert!(parse("{:0..32}").is_err());
        // just inside 32-bit range
        assert!(parse("{:0..31}").is_ok());

        // missing parts
        assert!(parse("{:0..4").is_err());
        assert!(parse("{:0..}").is_err());
        assert!(parse("{:..4}").is_err());
        assert!(parse("{:0.4}").is_err());
        assert!(parse("{:0...4}").is_err());
    }

    #[test]
    fn arrays() {
        assert_eq!(
            parse("{:[u8; 0]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(0),
            })])
        );

        // Space is optional.
        assert_eq!(
            parse("{:[u8;42]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(42),
            })])
        );

        // Multiple spaces are ok.
        assert_eq!(
            parse("{:[u8;    257]}"),
            Ok(vec![Fragment::Parameter(Parameter {
                index: 0,
                ty: Type::U8Array(257),
            })])
        );

        // No tabs or other whitespace.
        assert!(parse("{:[u8; \t 3]}").is_err());
        assert!(parse("{:[u8; \n 3]}").is_err());
        // Too large.
        assert!(parse("{:[u8; 9999999999999999999999999]}").is_err());
    }

    #[test]
    fn error_msg() {
        assert_eq!(
            parse("{:dunno}"),
            Err("malformed format string (invalid type specifier `dunno`)".into())
        );
        assert_eq!(
            parse("{}"),
            Err("malformed format string (missing `:`)".into())
        );
        assert_eq!(
            parse("{0}"),
            Err("malformed format string (missing `:`)".into())
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
