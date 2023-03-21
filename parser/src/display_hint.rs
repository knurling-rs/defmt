use std::str::FromStr;

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
        crate_name: String,
    },
    /// Display hints currently not supported / understood
    Unknown(String),
}

impl DisplayHint {
    /// Parses the display hint (e.g. the `#x` in `{=u8:#x}`)
    pub(crate) fn parse(mut s: &str) -> Option<Self> {
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
                [bitflags_name, package, disambiguator, crate_name] => {
                    return Some(DisplayHint::Bitflags {
                        name: bitflags_name.into(),
                        package: package.into(),
                        disambiguator: disambiguator.into(),
                        crate_name: crate_name.into(),
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
}

/// Precision of ISO8601 datetime
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Millis,
    Seconds,
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
