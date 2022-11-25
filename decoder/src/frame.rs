use std::{
    convert::TryFrom,
    fmt::{self, Write as _},
    mem,
};

use crate::{Arg, BitflagsKey, Table};
use chrono::TimeZone;
use colored::Colorize;
use defmt_parser::{DisplayHint, Fragment, Level, ParserMode, TimePrecision, Type};

/// Used to convert a `i128` value into right target type in hex
struct I128Hex(i128, Type);

impl std::fmt::LowerHex for I128Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Type::I8 => fmt::LowerHex::fmt(&(self.0 as i8), f),
            Type::I16 => fmt::LowerHex::fmt(&(self.0 as i16), f),
            Type::I32 => fmt::LowerHex::fmt(&(self.0 as i32), f),
            Type::I64 => fmt::LowerHex::fmt(&(self.0 as i64), f),
            Type::I128 => fmt::LowerHex::fmt(&(self.0 as i128), f),
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
            Type::I128 => fmt::UpperHex::fmt(&(self.0 as i128), f),
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

    pub fn level(&self) -> Option<Level> {
        self.level
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    fn format_args(&self, format: &str, args: &[Arg], parent_hint: Option<&DisplayHint>) -> String {
        self.format_args_real(format, args, parent_hint).unwrap() // cannot fail, we only write to a `String`
    }

    fn format_args_real(
        &self,
        format: &str,
        args: &[Arg],
        parent_hint: Option<&DisplayHint>,
    ) -> Result<String, fmt::Error> {
        let params = defmt_parser::parse(format, ParserMode::ForwardsCompatible).unwrap();
        let mut buf = String::new();
        for param in params {
            match param {
                Fragment::Literal(lit) => {
                    buf.push_str(&lit);
                }
                Fragment::Parameter(param) => {
                    let hint = param.hint.as_ref().or(parent_hint);

                    match &args[param.index] {
                        Arg::Bool(x) => write!(buf, "{}", x)?,
                        Arg::F32(x) => write!(buf, "{}", ryu::Buffer::new().format(*x))?,
                        Arg::F64(x) => write!(buf, "{}", ryu::Buffer::new().format(*x))?,
                        Arg::Uxx(x) => {
                            match param.ty {
                                Type::BitField(range) => {
                                    let left_zeroes =
                                        mem::size_of::<u128>() * 8 - range.end as usize;
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
                                        self.format_bytes(&bstr, hint, &mut buf)?
                                    } else {
                                        self.format_u128(bitfields as u128, hint, &mut buf)?;
                                    }
                                }
                                _ => match hint {
                                    Some(DisplayHint::ISO8601(precision)) => {
                                        self.format_iso8601(*x as u64, precision, &mut buf)?
                                    }
                                    Some(DisplayHint::Debug) => {
                                        self.format_u128(*x as u128, parent_hint, &mut buf)?
                                    }
                                    _ => self.format_u128(*x as u128, hint, &mut buf)?,
                                },
                            }
                        }
                        Arg::Ixx(x) => self.format_i128(*x as i128, param.ty, hint, &mut buf)?,
                        Arg::Str(x) | Arg::Preformatted(x) => self.format_str(x, hint, &mut buf)?,
                        Arg::IStr(x) => self.format_str(x, hint, &mut buf)?,
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
                                    if elements.iter().filter(|e| e.format == "{=u8}").count()
                                        != 0 =>
                                {
                                    let vals = elements
                                        .iter()
                                        .map(|e| match e.args.as_slice() {
                                            [Arg::Uxx(v)] => u8::try_from(*v)
                                                .expect("the value must be in u8 range"),
                                            _ => panic!(
                                                "FormatSlice should only contain one argument"
                                            ),
                                        })
                                        .collect::<Vec<u8>>();
                                    self.format_bytes(&vals, hint, &mut buf)?
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
                        Arg::Slice(x) => self.format_bytes(x, hint, &mut buf)?,
                        Arg::Char(c) => write!(buf, "{}", c)?,
                    }
                }
            }
        }
        Ok(buf)
    }

    fn format_u128(
        &self,
        x: u128,
        hint: Option<&DisplayHint>,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        match hint {
            Some(DisplayHint::NoHint { zero_pad }) => write!(buf, "{:01$}", x, zero_pad)?,
            Some(DisplayHint::Binary {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{:#01$b}", x, zero_pad)?,
                false => write!(buf, "{:01$b}", x, zero_pad)?,
            },
            Some(DisplayHint::Hexadecimal {
                uppercase,
                alternate,
                zero_pad,
            }) => match (alternate, uppercase) {
                (false, false) => write!(buf, "{:01$x}", x, zero_pad)?,
                (false, true) => write!(buf, "{:01$X}", x, zero_pad)?,
                (true, false) => write!(buf, "{:#01$x}", x, zero_pad)?,
                (true, true) => write!(buf, "{:#01$X}", x, zero_pad)?,
            },
            Some(DisplayHint::Microseconds) => {
                let seconds = x / 1_000_000;
                let micros = x % 1_000_000;
                write!(buf, "{}.{:06}", seconds, micros)?;
            }
            Some(DisplayHint::Bitflags {
                name,
                package,
                disambiguator,
            }) => {
                // The bitflags hint is only used internally, in `Format` impls generated by
                // `defmt::bitflags!`.
                let key = BitflagsKey {
                    ident: name.clone(),
                    package: package.clone(),
                    disambig: disambiguator.clone(),
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
                        write!(buf, "{}", x)?;
                    }
                }
            }
            _ => write!(buf, "{}", x)?,
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
            Some(DisplayHint::NoHint { zero_pad }) => write!(buf, "{:01$}", x, zero_pad)?,
            Some(DisplayHint::Binary {
                alternate,
                zero_pad,
            }) => match alternate {
                true => write!(buf, "{:#01$b}", x, zero_pad)?,
                false => write!(buf, "{:01$b}", x, zero_pad)?,
            },
            Some(DisplayHint::Hexadecimal {
                uppercase,
                alternate,
                zero_pad,
            }) => {
                let value = I128Hex(x, ty);
                match (alternate, uppercase) {
                    (false, false) => write!(buf, "{:01$x}", value, zero_pad)?,
                    (false, true) => write!(buf, "{:01$X}", value, zero_pad)?,
                    (true, false) => write!(buf, "{:#01$x}", value, zero_pad)?,
                    (true, true) => write!(buf, "{:#01$X}", value, zero_pad)?,
                }
            }
            _ => write!(buf, "{}", x)?,
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
                                write!(buf, "\\x{:02x}", byte).ok();
                            }
                        }
                    }
                }
                buf.push('\"');
            }
            Some(DisplayHint::Hexadecimal { .. }) | Some(DisplayHint::Binary { .. }) => {
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
            _ => write!(buf, "{:?}", bytes)?,
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
            write!(buf, "{:?}", s)?;
        } else {
            buf.push_str(s);
        }
        Ok(())
    }

    fn format_iso8601(
        &self,
        timestamp: u64,
        precision: &TimePrecision,
        buf: &mut String,
    ) -> Result<(), fmt::Error> {
        let format = match precision {
            TimePrecision::Millis => chrono::SecondsFormat::Millis,
            TimePrecision::Seconds => chrono::SecondsFormat::Secs,
        };
        let date_time = match precision {
            TimePrecision::Millis => chrono::Utc.timestamp_millis_opt(timestamp as i64),
            TimePrecision::Seconds => chrono::Utc.timestamp_opt(timestamp as i64, 0),
        }
        .unwrap();
        write!(buf, "{}", date_time.to_rfc3339_opts(format, true))
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
            format!("{} ", level)
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

        write!(f, "{}{}{}", timestamp, level, args)
    }
}
