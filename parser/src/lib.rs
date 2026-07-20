//! Parsing library for [`defmt`] format strings.
//!
//! This is an implementation detail of [`defmt`] and not meant to be consumed by other tools at the
//! moment so all the API is unstable.
//!
//! [`defmt`]: https://github.com/knurling-rs/defmt

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

mod display_hint;
#[cfg(test)]
mod tests;
mod types;

use std::{borrow::Cow, ops::Range};

pub use crate::{
    display_hint::{DisplayHint, TimePrecision},
    types::Type,
};

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
    #[error("invalid argument name `{0}`")]
    InvalidArgumentName(String),
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
    /// The argument name, if this parameter refers to a named argument (e.g. `{owner}`).
    ///
    /// Named arguments are assigned the indices following the positional arguments, in
    /// order of first appearance; `index` is always set accordingly.
    pub name: Option<String>,
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
/// argument := integer | identifier
/// identifier := [ 'a'-'z' | 'A'-'Z' | '_' ] [ 'a'-'z' | 'A'-'Z' | '0'-'9' | '_' ]*
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
    name: Option<String>,
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

/// Parse and consume an array at the beginning of `s`.
///
/// Return the length of the array.
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

    // First, optional argument: an integer index or an identifier (named argument).
    let mut index = None;
    let mut name = None;
    let index_end = input
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(input.len());

    if index_end != 0 {
        index = Some(input[..index_end].parse::<usize>()?);
        input = &input[index_end..];
    } else if input
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    {
        // Identifiers may not start with a digit; that case was handled above.
        let name_end = input
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(input.len());
        let ident = &input[..name_end];
        if ident == "_" {
            return Err(Error::InvalidArgumentName(ident.to_owned()));
        }
        name = Some(ident.to_owned());
        input = &input[name_end..];
    }

    // Then, optional type
    let mut ty = Type::default(); // when no explicit type; use the default one

    if input.starts_with(TYPE_PREFIX) {
        // skip the prefix
        input = &input[TYPE_PREFIX.len()..];

        // type is delimited by `HINT_PREFIX` or end-of-string
        let type_end = input.find(HINT_PREFIX).unwrap_or(input.len());
        let type_fragment = &input[..type_end];

        const FORMAT_ARRAY_START: &str = "[?;";
        const U8_ARRAY_START: &str = "[u8;";

        // what comes next is the type
        ty = if let Ok(ty) = type_fragment.parse() {
            Ok(ty)
        } else if let Some(s) = type_fragment.strip_prefix(U8_ARRAY_START) {
            Ok(Type::U8Array(parse_array(s)?))
        } else if let Some(s) = type_fragment.strip_prefix(FORMAT_ARRAY_START) {
            Ok(Type::FormatArray(parse_array(s)?))
        } else if let Some((range, used)) = parse_range(type_fragment) {
            // Check for bitfield syntax.
            match used != type_fragment.len() {
                true => Err(Error::TrailingDataAfterBitfieldRange),
                false => Ok(Type::BitField(range)),
            }
        } else {
            Err(Error::InvalidTypeSpecifier(input.to_owned()))
        }?;

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

        hint = match (DisplayHint::parse(input), mode) {
            (Some(a), _) => Some(a),
            (None, ParserMode::Strict) => return Err(Error::UnknownDisplayHint(input.to_owned())),
            (None, ParserMode::ForwardsCompatible) => Some(DisplayHint::Unknown(input.to_owned())),
        };
    } else if !input.is_empty() {
        return Err(Error::UnexpectedContentInFormatString(input.to_owned()));
    }

    Ok(Param {
        index,
        name,
        ty,
        hint,
    })
}

fn unescape_literal(unescaped_literal: &str) -> Result<Cow<'_, str>, Error> {
    // Replace `{{` with `{` and `}}` with `}`. Single braces are errors.

    // Scan for single braces first. The rest is trivial.
    let mut last_open = false;
    let mut last_close = false;
    for c in unescaped_literal.chars() {
        match c {
            '{' => last_open = !last_open,
            '}' => last_close = !last_close,
            _ if last_open => return Err(Error::UnmatchedOpenBracket),
            _ if last_close => return Err(Error::UnmatchedCloseBracket),
            _ => {}
        }
    }

    // Handle trailing unescaped `{` or `}`.
    if last_open {
        return Err(Error::UnmatchedOpenBracket);
    } else if last_close {
        return Err(Error::UnmatchedCloseBracket);
    }

    // FIXME: This always allocates a `String`, so the `Cow` is useless.
    Ok(unescaped_literal
        .replace("{{", "{")
        .replace("}}", "}")
        .into())
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
    /// A fragment whose parameters do not have their final argument index assigned yet.
    enum RawFragment<'f> {
        Literal(Cow<'f, str>),
        Parameter(Param),
    }

    let mut raw_fragments = Vec::new();

    // Index after the `}` of the last format specifier.
    let mut end_pos = 0;

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
            raw_fragments.push(RawFragment::Literal(unescape_literal(unescaped_literal)?));
        }

        // Else, this is a format specifier. It ends at the next `}`.
        let len = chars
            .as_str()
            .find('}')
            .ok_or(Error::UnmatchedOpenBracket)?;
        end_pos = brace_pos + 1 + len + 1;

        // Parse the contents inside the braces.
        let param_str = &format_string[brace_pos + 1..][..len];
        raw_fragments.push(RawFragment::Parameter(parse_param(param_str, mode)?));
    }

    // Trailing literal.
    if end_pos != format_string.len() {
        raw_fragments.push(RawFragment::Literal(unescape_literal(
            &format_string[end_pos..],
        )?));
    }

    // Assign argument indices.
    //
    // Parameters with an explicit index (`{0}`) use it as-is and parameters with neither an
    // index nor a name (`{}`) take the next implicit index, exactly like `core::format_args!`.
    // Named parameters (`{owner}`) are assigned the indices *after* all positional ones, in
    // order of first appearance, mirroring how `core::format_args!` appends named arguments
    // after the positional ones.
    //
    // NOTE: this assignment is recomputed from the interned format string when decoding logs,
    // so it must stay deterministic: `defmt-macros` serializes the arguments in this order at
    // compile time and `defmt-decoder` derives the same order from the same string at decode
    // time (both through this function).
    let mut implicit_count = 0;
    let mut max_explicit = None;
    let mut names: Vec<&str> = Vec::new();
    for frag in &raw_fragments {
        if let RawFragment::Parameter(param) = frag {
            match (&param.index, &param.name) {
                (Some(index), _) => max_explicit = max_explicit.max(Some(*index)),
                (None, Some(name)) => {
                    if !names.iter().any(|n| n == name) {
                        names.push(name);
                    }
                }
                (None, None) => implicit_count += 1,
            }
        }
    }
    let positional_count = implicit_count.max(max_explicit.map_or(0, |max| max + 1));
    let names = names.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    let mut next_arg_index = 0;
    let mut fragments = Vec::new();
    for frag in raw_fragments {
        match frag {
            RawFragment::Literal(literal) => fragments.push(Fragment::Literal(literal)),
            RawFragment::Parameter(param) => {
                let index = match (param.index, &param.name) {
                    (Some(index), _) => index,
                    (None, Some(name)) => {
                        let name_index =
                            names.iter().position(|n| n == name).unwrap(/* collected above */);
                        positional_count + name_index
                    }
                    (None, None) => {
                        // If there is no explicit index, assign the next one.
                        let index = next_arg_index;
                        next_arg_index += 1;
                        index
                    }
                };
                fragments.push(Fragment::Parameter(Parameter {
                    index,
                    ty: param.ty,
                    hint: param.hint,
                    name: param.name,
                }));
            }
        }
    }

    // Check for argument type conflicts.
    let mut args = Vec::new();
    for frag in &fragments {
        if let Fragment::Parameter(Parameter { index, ty, .. }) = frag {
            if args.len() <= *index {
                args.resize(*index + 1, None);
            }

            match &args[*index] {
                None => args[*index] = Some(ty.clone()),
                Some(other_ty) => match (other_ty, ty) {
                    (Type::BitField(_), Type::BitField(_)) => {} // FIXME: Bitfield range shouldn't be part of the type.
                    (a, b) if a != b => {
                        return Err(Error::ConflictingTypes(*index, a.clone(), b.clone()))
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
