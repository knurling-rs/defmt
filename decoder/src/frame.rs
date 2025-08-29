use std::{
    convert::TryFrom,
    fmt::{self, Write as _},
    mem,
};

use crate::{Arg, BitflagsKey, Table};
use colored::Colorize;
use defmt_parser::{DisplayHint, Fragment, Level, ParserMode, TimePrecision, Type};
use time::{macros::format_description, OffsetDateTime};

/// Used to convert a `i128` value into right target type in hex
struct I128Hex(i128, Type);

impl std::fmt::LowerHex for I128Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Type::I8 => fmt::LowerHex::fmt(&(self.0 as i8), f),
            Type::I16 => fmt::LowerHex::fmt(&(self.0 as i16), f),
            Type::I32 => fmt::LowerHex::fmt(&(self.0 as i32), f),
            Type::I64 => fmt::LowerHex::fmt(&(self.0 as i64), f),
            Type::I128 => fmt::LowerHex::fmt(&self.0, f),
            _ => panic!("Unsupported type '{:?}' found.", self.1),
        }
    }
}

impl std::fmt::UpperHex for I128Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Type::I8 => fmt::UpperHex::fmt(&(self.0 as i8), f),
            Type::I16 => fmt::UpperHex::fmt(&(self.0 as i16), f),
            Type::I32 => fmt::UpperHex::fmt(&(self.0 as i32), f),
            Type::I64 => fmt::UpperHex::fmt(&(self.0 as i64), f),
            Type::I128 => fmt::UpperHex::fmt(&self.0, f),
            _ => panic!("Unsupported type '{:?}' found.", self.1),
        }
    }
}

/// A log frame
#[derive(Debug, PartialEq)]
pub struct Frame<'t> {
    table: &'t Table,
    level: Option<Level>,
    index: u64,
    timestamp_format: Option<&'t str>,
    timestamp_args: Vec<Arg<'t>>,
    // Format string
    format: &'t str,
    args: Vec<Arg<'t>>,
}

impl<'t> Frame<'t> {
    pub(crate) fn new(
        table: &'t Table,
        level: Option<Level>,
        index: u64,
        timestamp_format: Option<&'t str>,
        timestamp_args: Vec<Arg<'t>>,
        format: &'t str,
        args: Vec<Arg<'t>>,
    ) -> Self {
        Self {
            table,
            level,
            index,
            timestamp_format,
            timestamp_args,
            format,
            args,
        }
    }

    /// Returns a struct that will format this log frame (including message, timestamp, level,
    /// etc.).
    pub fn display(&'t self, colored: bool) -> DisplayFrame<'t> {
        DisplayFrame {
            frame: self,
            colored,
        }
    }

    pub fn display_timestamp(&'t self) -> Option<DisplayTimestamp<'t>> {
        self.timestamp_format
            .map(|_| DisplayTimestamp { frame: self })
    }

    /// Returns a struct that will format the message contained in this log frame.
    pub fn display_message(&'t self) -> DisplayMessage<'t> {
        DisplayMessage { frame: self }
    }

    /// Returns an iterator over the fragments of the message contained in this log frame.
    ///
    /// Collecting this into a String will yield the same result as [`Self::display_message`], but
    /// this iterator will yield interpolated fragments on their own. For example, the log:
    ///
    /// ```ignore
    /// defmt::info!("foo = {}, bar = {}", 1, 2);
    /// ```
    ///
    /// Will yield the following strings:
    ///
    /// ```ignore
    /// vec!["foo = ", "1", ", bar = ", "2"]
    /// ```
    ///
    /// Note that nested fragments will not yield separately:
    ///
    /// ```ignore
    /// defmt::info!("foo = {}", Foo { bar: 1 });
    /// ```
    ///
    /// Will yield:
    ///
    /// ```ignore
    /// vec!["foo = ", "Foo { bar: 1 }"]
    /// ```
    ///
    /// This iterator yields the same fragments as [`Self::fragments`], so you can zip them
    /// together to get both representations.
    pub fn display_fragments(&'t self) -> DisplayFragments<'t> {
        DisplayFragments {
            frame: self,
            iter: self.fragments().into_iter(),
        }
    }

    /// Returns the fragments of the message contained in this log frame.
    ///
    /// Each fragment represents a part of the log message. See [`Fragment`] for more details.
    ///
    /// This iterator yields the same fragments as [`Self::display_fragments`], so you can zip them
    /// together to get both representations.
    pub fn fragments(&'t self) -> Vec<Fragment<'t>> {
        defmt_parser::parse(self.format, ParserMode::ForwardsCompatible).unwrap()
    }

    pub fn level(&self) -> Option<Level> {
        self.level
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    fn format_args(&self, format: &str, args: &[Arg], parent_hint: Option<&DisplayHint>) -> String {
        let params = defmt_parser::parse(format, ParserMode::ForwardsCompatible).unwrap();
        let mut buf = String::new();
        for param in params {
            self.format_fragment(param, &mut buf, args, parent_hint)
                .unwrap(); // cannot fail, we only write to a `String`
        }
        buf
    }

    fn format_fragment(
        &self,
        param: Fragment<'_>,
        buf: &mut String,
        args: &[Arg],
        parent_hint: Option<&DisplayHint>,
    ) -> Result<(), fmt::Error> {
        match param {
            Fragment::Literal(lit) => {
                buf.push_str(&lit);
            }
            Fragment::Parameter(param) => {
                let hint = param.hint.as_ref().or(parent_hint);

                match &args[param.index] {
                    Arg::Bool(x) => write!(buf, "{x}")?,
                    Arg::F32(x) => write!(buf, "{}", ryu::Buffer::new().format(*x))?,
                    Arg::F64(x) => write!(buf, "{}", ryu::Buffer::new().format(*x))?,
                    Arg::Uxx(x) => {
                        match param.ty {
                            Type::BitField(range) => {
                                let left_zeroes = mem::size_of::<u128>() * 8 - range.end as usize;
                                let right_zeroes = left_zeroes + range.start as usize;
                                // isolate the desired bitfields
                                let bitfields = (*x << left_zeroes) >> right_zeroes;

                                if let Some(DisplayHint::Ascii) = hint {
                                    let bstr = bitfields
                                        .to_be_bytes()
                                        .iter()
                                        .skip(right_zeroes / 8)
                                        .copied()
                                        .collect::<Vec<u8>>();
                                    self.format_bytes(&bstr, hint, buf)?
                                } else {
                                    self.format_u128(bitfields, hint, buf)?;
                                }
                            }
                            _ => match hint {
                                Some(DisplayHint::ISO8601(precision)) => {
                                    self.format_iso8601(*x as u64, precision, buf)?
                                }
                                Some(DisplayHint::Debug) => {
                                    self.format_u128(*x, parent_hint, buf)?
                                }
                                _ => self.format_u128(*x, hint, buf)?,
                            },
                        }
                    }
                    Arg::Ixx(x) => self.format_i128(*x, param.ty, hint, buf)?,
                    Arg::Str(x) | Arg::Preformatted(x) => self.format_str(x, hint, buf)?,
                    Arg::IStr(x) => self.format_str(x, hint, buf)?,
                    Arg::Format { format, args } => match parent_hint {
                        Some(DisplayHint::Ascii) => {
                            buf.push_str(&self.format_args(format, args, parent_hint));
                        }
                        _ => buf.push_str(&self.format_args(format, args, hint)),
                    },
                    Arg::FormatSequence { args } => {
                        for arg in args {
                            buf.push_str(&self.format_args("{=?}", &[arg.clone()], hint))
                        }
                    }
                    Arg::FormatSlice { elements } => {
                        match hint {
                            // Filter Ascii Hints, which contains u8 byte slices
                            Some(DisplayHint::Ascii)
                                if elements.iter().filter(|e| e.format == "{=u8}").count() != 0 =>
                            {
                                let vals = elements
                                    .iter()
                                    .map(|e| match e.args.as_slice() {
                                        [Arg::Uxx(v)] => {
                                            u8::try_from(*v).expect("the value must be in u8 range")
                                        }
                                        _ => panic!("FormatSlice should only contain one argument"),
                                    })
                                    .collect::<Vec<u8>>();
                                self.format_bytes(&vals, hint, buf)?
                            }
                            _ => {
                                buf.write_str("[")?;
                                let mut is_first = true;
                                for element in elements {
                                    if !is_first {
                                        buf.write_str(", ")?;
                                    }
                                    is_first = false;
                                    buf.write_str(&self.format_args(
                                        element.format,
                                        &element.args,
                                        hint,
                                    ))?;
                                }
                                buf.write_str("]")?;
                            }
                        }
                    }
                    Arg::Slice(x) => self.format_bytes(x, hint, buf)?,
                    Arg::Char(c) => write!(buf, "{c}")?,
                }
            }
        }

        Ok(())
    }

    fn format_u128(
        &self,
        x: u128,
        hint: Option<&DisplayHint>,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        match hint {
            Some(DisplayHint::NoHint { zero_pad }) => write!(buf, "{x:0zero_pad$}")?,
            Some(DisplayHint::Binary {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{x:#0zero_pad$b}")?,
                false => write!(buf, "{x:0zero_pad$b}")?,
            },
            Some(DisplayHint::Octal {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{x:#0zero_pad$o}")?,
                false => write!(buf, "{x:0zero_pad$o}")?,
            },
            Some(DisplayHint::Hexadecimal {
                uppercase,
                alternate,
                zero_pad,
            }) => match (alternate, uppercase) {
                (false, false) => write!(buf, "{x:0zero_pad$x}")?,
                (false, true) => write!(buf, "{x:0zero_pad$X}")?,
                (true, false) => write!(buf, "{x:#0zero_pad$x}")?,
                (true, true) => write!(buf, "{x:#0zero_pad$X}")?,
            },
            Some(DisplayHint::Seconds(TimePrecision::Micros)) => {
                let seconds = x / 1_000_000;
                let micros = x % 1_000_000;
                write!(buf, "{seconds}.{micros:06}")?;
            }
            Some(DisplayHint::Seconds(TimePrecision::Millis)) => {
                let seconds = x / 1_000;
                let millis = x % 1_000;
                write!(buf, "{seconds}.{millis:03}")?;
            }
            Some(DisplayHint::Time(TimePrecision::Micros)) => {
                self.format_time(x, &TimePrecision::Micros, buf)?;
            }
            Some(DisplayHint::Time(TimePrecision::Millis)) => {
                self.format_time(x, &TimePrecision::Millis, buf)?;
            }
            Some(DisplayHint::Time(TimePrecision::Seconds)) => {
                self.format_time(x, &TimePrecision::Seconds, buf)?;
            }
            Some(DisplayHint::Bitflags {
                name,
                package,
                disambiguator,
                crate_name,
            }) => {
                // The bitflags hint is only used internally, in `Format` impls generated by
                // `defmt::bitflags!`.
                let key = BitflagsKey {
                    ident: name.clone(),
                    package: package.clone(),
                    disambig: disambiguator.clone(),
                    crate_name: crate_name.clone(),
                };
                match self.table.bitflags.get(&key) {
                    Some(flags) => {
                        let set_flags = flags
                            .iter()
                            .filter(|(_, value)| {
                                if *value == 0 && x != 0 {
                                    false
                                } else {
                                    x & value == *value
                                }
                            })
                            .map(|(name, _)| name.clone())
                            .collect::<Vec<_>>();
                        if set_flags.is_empty() {
                            write!(buf, "(empty)")?;
                        } else {
                            write!(buf, "{}", set_flags.join(" | "))?;
                        }
                    }
                    None => {
                        // FIXME return an internal error here
                        write!(buf, "{x}")?;
                    }
                }
            }
            _ => write!(buf, "{x}")?,
        }
        Ok(())
    }

    fn format_i128(
        &self,
        x: i128,
        ty: Type,
        hint: Option<&DisplayHint>,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        match hint {
            Some(DisplayHint::NoHint { zero_pad }) => write!(buf, "{x:0zero_pad$}")?,
            Some(DisplayHint::Binary {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{x:#0zero_pad$b}")?,
                false => write!(buf, "{x:0zero_pad$b}")?,
            },
            Some(DisplayHint::Octal {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{x:#0zero_pad$o}")?,
                false => write!(buf, "{x:0zero_pad$o}")?,
            },
            Some(DisplayHint::Hexadecimal {
                uppercase,
                alternate,
                zero_pad,
            }) => {
                let value = I128Hex(x, ty);
                match (alternate, uppercase) {
                    (false, false) => write!(buf, "{value:0zero_pad$x}")?,
                    (false, true) => write!(buf, "{value:0zero_pad$X}")?,
                    (true, false) => write!(buf, "{value:#0zero_pad$x}")?,
                    (true, true) => write!(buf, "{value:#0zero_pad$X}")?,
                }
            }
            _ => write!(buf, "{x}")?,
        }
        Ok(())
    }

    fn format_bytes(
        &self,
        bytes: &[u8],
        hint: Option<&DisplayHint>,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        match hint {
            Some(DisplayHint::Ascii) => {
                // byte string literal syntax: b"Hello\xffworld"
                buf.push_str("b\"");
                for byte in bytes {
                    match byte {
                        // special escaping
                        b'\t' => buf.push_str("\\t"),
                        b'\n' => buf.push_str("\\n"),
                        b'\r' => buf.push_str("\\r"),
                        b' ' => buf.push(' '),
                        b'\"' => buf.push_str("\\\""),
                        b'\\' => buf.push_str("\\\\"),
                        _ => {
                            if byte.is_ascii_graphic() {
                                buf.push(*byte as char);
                            } else {
                                // general escaped form
                                write!(buf, "\\x{byte:02x}").ok();
                            }
                        }
                    }
                }
                buf.push('\"');
            }
            Some(DisplayHint::Hexadecimal { .. })
            | Some(DisplayHint::Octal { .. })
            | Some(DisplayHint::Binary { .. }) => {
                // `core::write!` doesn't quite produce the output we want, for example
                // `write!("{:#04x?}", bytes)` produces a multi-line output
                // `write!("{:02x?}", bytes)` is single-line but each byte doesn't include the "0x" prefix
                buf.push('[');
                let mut is_first = true;
                for byte in bytes {
                    if !is_first {
                        buf.push_str(", ");
                    }
                    is_first = false;
                    self.format_u128(*byte as u128, hint, buf)?;
                }
                buf.push(']');
            }
            Some(DisplayHint::Cbor) => {
                use core::fmt::Write;
                let parsed = cbor_edn::Sequence::from_cbor(bytes);
                match parsed {
                    Ok(parsed) => buf.write_str(&parsed.serialize())?,
                    Err(err) => {
                        write!(buf, "invalid CBOR (error: {}, bytes: {:02x?})", err, bytes)?
                    }
                }
            }
            _ => write!(buf, "{bytes:?}")?,
        }
        Ok(())
    }

    fn format_str(
        &self,
        s: &str,
        hint: Option<&DisplayHint>,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        if hint == Some(&DisplayHint::Debug) {
            write!(buf, "{s:?}")?;
        } else {
            buf.push_str(s);
        }
        Ok(())
    }

    fn format_time(
        &self,
        timestamp: u128,
        precision: &TimePrecision,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        let div_rem = |x, y| (x / y, x % y);

        let (timestamp, decimals) = match precision {
            TimePrecision::Micros => div_rem(timestamp, 1_000_000),
            TimePrecision::Millis => div_rem(timestamp, 1_000),
            TimePrecision::Seconds => (timestamp, 0),
        };

        let (timestamp, seconds) = div_rem(timestamp, 60);
        let (timestamp, minutes) = div_rem(timestamp, 60);
        let (timestamp, hours) = div_rem(timestamp, 24);
        let days = timestamp;

        if days == 0 {
            match precision {
                TimePrecision::Micros => write!(
                    buf,
                    "{hours:0>2}:{minutes:0>2}:{seconds:0>2}.{decimals:0>6}"
                ),
                TimePrecision::Millis => write!(
                    buf,
                    "{hours:0>2}:{minutes:0>2}:{seconds:0>2}.{decimals:0>3}"
                ),
                TimePrecision::Seconds => write!(buf, "{hours:0>2}:{minutes:0>2}:{seconds:0>2}"),
            }
        } else {
            match precision {
                TimePrecision::Micros => write!(
                    buf,
                    "{days}:{hours:0>2}:{minutes:0>2}:{seconds:0>2}.{decimals:0>6}"
                ),
                TimePrecision::Millis => write!(
                    buf,
                    "{days}:{hours:0>2}:{minutes:0>2}:{seconds:0>2}.{decimals:0>3}"
                ),
                TimePrecision::Seconds => {
                    write!(buf, "{days}:{hours:0>2}:{minutes:0>2}:{seconds:0>2}")
                }
            }
        }
    }

    fn format_iso8601(
        &self,
        timestamp: u64,
        precision: &TimePrecision,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        let format = match precision {
            TimePrecision::Micros => format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:6]Z"
            ),
            TimePrecision::Millis => format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
            ),
            TimePrecision::Seconds => {
                format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
            }
        };
        let date_time = OffsetDateTime::from_unix_timestamp_nanos(match precision {
            TimePrecision::Micros => timestamp as i128 * 1_000,
            TimePrecision::Millis => timestamp as i128 * 1_000_000,
            TimePrecision::Seconds => timestamp as i128 * 1_000_000_000,
        })
        .unwrap();
        write!(buf, "{}", date_time.format(format).unwrap())
    }
}

pub struct DisplayTimestamp<'t> {
    frame: &'t Frame<'t>,
}

impl fmt::Display for DisplayTimestamp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self.frame.format_args(
            self.frame.timestamp_format.unwrap(),
            &self.frame.timestamp_args,
            None,
        );
        f.write_str(&args)
    }
}

pub struct DisplayMessage<'t> {
    frame: &'t Frame<'t>,
}

impl fmt::Display for DisplayMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = self
            .frame
            .format_args(self.frame.format, &self.frame.args, None);
        f.write_str(&args)
    }
}

/// An iterator over the fragments of a log message, formatted as strings.
///
/// See [`Frame::display_fragments`].
pub struct DisplayFragments<'t> {
    frame: &'t Frame<'t>,
    iter: std::vec::IntoIter<Fragment<'t>>,
}

impl Iterator for DisplayFragments<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        self.frame
            .format_fragment(self.iter.next()?, &mut buf, &self.frame.args, None)
            .ok()?;
        Some(buf)
    }
}

/// Prints a `Frame` when formatted via `fmt::Display`, including all included metadata (level,
/// timestamp, ...).
pub struct DisplayFrame<'t> {
    frame: &'t Frame<'t>,
    colored: bool,
}

impl fmt::Display for DisplayFrame<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = if let Some(level) = self.frame.level {
            let level = if self.colored {
                match level {
                    Level::Trace => "TRACE".dimmed().to_string(),
                    Level::Debug => "DEBUG".normal().to_string(),
                    Level::Info => "INFO".green().to_string(),
                    Level::Warn => "WARN".yellow().to_string(),
                    Level::Error => "ERROR".red().to_string(),
                }
            } else {
                match level {
                    Level::Trace => "TRACE".to_string(),
                    Level::Debug => "DEBUG".to_string(),
                    Level::Info => "INFO".to_string(),
                    Level::Warn => "WARN".to_string(),
                    Level::Error => "ERROR".to_string(),
                }
            };
            format!("{level} ")
        } else {
            "".to_string()
        };

        let timestamp = self
            .frame
            .timestamp_format
            .map(|fmt| {
                format!(
                    "{} ",
                    self.frame
                        .format_args(fmt, &self.frame.timestamp_args, None,),
                )
            })
            .unwrap_or_default();

        let args = self
            .frame
            .format_args(self.frame.format, &self.frame.args, None);

        write!(f, "{timestamp}{level}{args}")
    }
}
