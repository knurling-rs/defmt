use std::str::FromStr;

/// All display hints
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
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
    /// `:o`
    Octal {
        alternate: bool,
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
    /// `:us` `:ms`, formats integers as timestamps in seconds
    Seconds(TimePrecision),
    /// `:tus` `:tms` `:ts`, formats integers as human-readable time
    Time(TimePrecision),
    /// `:iso8601{ms,s}`, formats integers as timestamp in ISO8601 date time format
    ISO8601(TimePrecision),
    /// `__internal_bitflags_NAME` instructs the decoder to print the flags that are set, instead of
    /// the raw value.
    Bitflags {
        name: String,
        package: String,
        disambiguator: String,
        crate_name: Option<String>,
    },
    /// `:cbor`: There is CBOR data encoded in those bytes, to be shown in diagnostic notation.
    ///
    /// Technically, the byte string interpreted as a CBOR sequence, and shown in the diagnostic
    /// notation of a sequence. That is identical to processing a single CBOR item if there is just
    /// one present, but also allows reporting multiple items from consecutive memory; diagnostic
    /// notation turns those into comma separated items.
    // Should we have a flag to say "do a more display style stringification" (like, if you
    // recognize `54([64,h'20010db8'])`, don't show "IP'2001:db8::/64'" but just "2001:db8::64")?
    // Should we allow additional params that give a CDDL that further guides processing (like,
    // when data is not tagged but the shape is known for processing anyway)?
    Cbor,
    /// Display hints currently not supported / understood
    Unknown(String),
}

// https://github.com/rust-lang/rust/blob/99317ef14d0be42fa4039eea7c5ce50cb4e9aee7/library/core/src/fmt/mod.rs#L25
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Alignment {
    /// Indication that contents should be left-aligned.
    Left,

    /// Indication that contents should be right-aligned.
    Right,

    /// Indication that contents should be center-aligned.
    Center,
}
impl std::convert::TryFrom<char> for Alignment {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '<' => Ok(Alignment::Left),
            '^' => Ok(Alignment::Center),
            '>' => Ok(Alignment::Right),
            _ => Err(()),
        }
    }
}

impl DisplayHint {
    /// Parses the display hint (e.g. the `#x` in `{=u8:#x}`)
    pub(crate) fn parse(mut s: &str) -> Option<Self> {
        const BITFLAGS_HINT_START: &str = "__internal_bitflags_";

        // https://doc.rust-lang.org/std/fmt/index.html#syntax
        // format_spec := [[fill]align][sign]['#']['0'][width]['.' precision]type

        let mut align = Alignment::Right; // Default alignment is to the right.
        let mut fill: Option<char> = None;

        // If "[fill]align" is specified, that goes before everything else.
        // We can try to parse the fill and align, if it is filled with 0 and right aligned it can be treated as a
        // format we can support, supporting other fill & aligns requires modifying the DisplayHint struct and will be
        // a breaking change.
        if s.len() > 1 {
            let first_char = s.chars().next()?;
            if let Ok(align_specification) = Alignment::try_from(first_char) {
                // String starts with alignment character.
                align = align_specification;
                fill = Some(' '); // This is the default if no fill is specified.
                s = s.split_at(1).1;
            } else {
                if s.len() > 2 {
                    // String may start with fill, followed by alignment character.
                    let second_char = s.chars().skip(1).next()?;
                    if let Ok(align_specification) = Alignment::try_from(second_char) {
                        align = align_specification;
                        fill = Some(first_char);
                        s = s.split_at(2).1;
                    }
                }
            }
        }

        // If a fill/align was specified, check if it can be handled.
        if let Some(fill) = fill {
            if fill != '0' {
                // We can't handle this without breaking changes to the DisplayHint enum, so we fail.
                return None;
            }
        }

        if align != Alignment::Right {
            // We can't handle this without breaking changes to the DisplayHint enum, so we fail.
            return None;
        }

        // The `#` comes before any padding hints (I think this matches core::fmt).
        // It is ignored for types that don't have an alternate representation.
        let alternate = if let Some(rest) = s.strip_prefix('#') {
            s = rest;
            true
        } else {
            false
        };

        // If fill isn't specified yet, we may need to read the '0' with strip_prefix here.
        let stripped_prefix = if fill.is_some() {
            Some(s)
        } else {
            s.strip_prefix('0')
        };

        let zero_pad = if let Some(rest) = stripped_prefix {
            let (rest, columns) = parse_integer::<usize>(rest)?;
            s = rest;
            columns
        } else {
            0 // default behavior is the same as no zero-padding.
        };

        if let Some(stripped) = s.strip_prefix(BITFLAGS_HINT_START) {
            let parts = stripped.split('@').collect::<Vec<_>>();
            if parts.len() < 3 || parts.len() > 4 {
                return Some(DisplayHint::Unknown(s.into()));
            }
            return Some(DisplayHint::Bitflags {
                name: parts[0].into(),
                package: parts[1].into(),
                disambiguator: parts[2].into(),
                // crate_name was added in wire format version 4
                crate_name: parts.get(3).map(|&s| s.to_string()),
            });
        }

        Some(match s {
            "" => DisplayHint::NoHint { zero_pad },
            "us" => DisplayHint::Seconds(TimePrecision::Micros),
            "ms" => DisplayHint::Seconds(TimePrecision::Millis),
            "tus" => DisplayHint::Time(TimePrecision::Micros),
            "tms" => DisplayHint::Time(TimePrecision::Millis),
            "ts" => DisplayHint::Time(TimePrecision::Seconds),
            "a" => DisplayHint::Ascii,
            "b" => DisplayHint::Binary {
                alternate,
                zero_pad,
            },
            "o" => DisplayHint::Octal {
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
            "cbor" => DisplayHint::Cbor,
            "?" => DisplayHint::Debug,
            _ => return None,
        })
    }
}

/// Precision of timestamp
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimePrecision {
    Micros,
    Millis,
    Seconds,
}

/// Parses an integer at the beginning of `s`.
///
/// Returns the integer and remaining text, if `s` started with an integer. Any errors parsing the
/// number (which we already know only contains digits) are silently ignored.
fn parse_integer<T: FromStr>(s: &str) -> Option<(&str, T)> {
    let start_digits = s
        .as_bytes()
        .iter()
        .copied()
        .take_while(|b| b.is_ascii_digit())
        .count();
    let num = s[..start_digits].parse().ok()?;
    Some((&s[start_digits..], num))
}
