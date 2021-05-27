use std::{
    convert::TryFrom,
    fmt::{self, Write as _},
    mem,
};

use crate::Arg;
use colored::Colorize;
use defmt_parser::{DisplayHint, Fragment, Level, ParserMode, Type};

/// A log frame
#[derive(Debug, PartialEq)]
pub struct Frame<'t> {
    level: Level,
    index: u64,
    timestamp_format: Option<&'t str>,
    timestamp_args: Vec<Arg<'t>>,
    // Format string
    format: &'t str,
    args: Vec<Arg<'t>>,
}

impl<'t> Frame<'t> {
    pub(crate) fn new(
        level: Level,
        index: u64,
        timestamp_format: Option<&'t str>,
        timestamp_args: Vec<Arg<'t>>,
        format: &'t str,
        args: Vec<Arg<'t>>,
    ) -> Self {
        Self {
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

    pub fn display_timestamp(&'t self) -> Option<DisplayMessage<'t>> {
        self.timestamp_format.map(|fmt| DisplayMessage {
            format: fmt,
            args: &self.timestamp_args,
        })
    }

    /// Returns a struct that will format the message contained in this log frame.
    pub fn display_message(&'t self) -> DisplayMessage<'t> {
        DisplayMessage {
            format: self.format,
            args: &self.args,
        }
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

pub struct DisplayMessage<'t> {
    format: &'t str,
    args: &'t [Arg<'t>],
}

impl fmt::Display for DisplayMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = format_args(self.format, self.args, None);
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
        let level = if self.colored {
            match self.frame.level {
                Level::Trace => "TRACE".dimmed().to_string(),
                Level::Debug => "DEBUG".normal().to_string(),
                Level::Info => "INFO".green().to_string(),
                Level::Warn => "WARN".yellow().to_string(),
                Level::Error => "ERROR".red().to_string(),
            }
        } else {
            match self.frame.level {
                Level::Trace => "TRACE".to_string(),
                Level::Debug => "DEBUG".to_string(),
                Level::Info => "INFO".to_string(),
                Level::Warn => "WARN".to_string(),
                Level::Error => "ERROR".to_string(),
            }
        };

        let timestamp = self
            .frame
            .timestamp_format
            .map(|fmt| format!("{} ", format_args(&fmt, &self.frame.timestamp_args, None,)))
            .unwrap_or_default();
        let args = format_args(&self.frame.format, &self.frame.args, None);

        write!(f, "{}{} {}", timestamp, level, args)
    }
}

fn format_args(format: &str, args: &[Arg], parent_hint: Option<&DisplayHint>) -> String {
    format_args_real(format, args, parent_hint).unwrap() // cannot fail, we only write to a `String`
}

fn format_args_real(
    format: &str,
    args: &[Arg],
    parent_hint: Option<&DisplayHint>,
) -> Result<String, fmt::Error> {
    fn format_u128(
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
            _ => write!(buf, "{}", x)?,
        }
        Ok(())
    }

    fn format_i128(
        x: i128,
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
            _ => write!(buf, "{}", x)?,
        }
        Ok(())
    }

    fn format_bytes(
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
                    format_u128(*byte as u128, hint, buf)?;
                }
                buf.push(']');
            }
            _ => write!(buf, "{:?}", bytes)?,
        }
        Ok(())
    }

    fn format_str(s: &str, hint: Option<&DisplayHint>, buf: &mut String) -> Result<(), fmt::Error> {
        if hint == Some(&DisplayHint::Debug) {
            write!(buf, "{:?}", s)?;
        } else {
            buf.push_str(s);
        }
        Ok(())
    }

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
                                    format_bytes(&bstr, hint, &mut buf)?
                                } else {
                                    format_u128(bitfields as u128, hint, &mut buf)?;
                                }
                            }
                            _ => match hint {
                                Some(DisplayHint::Debug) => {
                                    format_u128(*x as u128, parent_hint, &mut buf)?
                                }
                                _ => format_u128(*x as u128, hint, &mut buf)?,
                            },
                        }
                    }
                    Arg::Ixx(x) => format_i128(*x as i128, hint, &mut buf)?,
                    Arg::Str(x) | Arg::Preformatted(x) => format_str(x, hint, &mut buf)?,
                    Arg::IStr(x) => format_str(x, hint, &mut buf)?,
                    Arg::Format { format, args } => buf.push_str(&format_args(format, args, hint)),
                    Arg::FormatSequence { args } => {
                        for arg in args {
                            buf.push_str(&format_args("{=?}", &[arg.clone()], hint))
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
                                format_bytes(&vals, hint, &mut buf)?
                            }
                            _ => {
                                buf.write_str("[")?;
                                let mut is_first = true;
                                for element in elements {
                                    if !is_first {
                                        buf.write_str(", ")?;
                                    }
                                    is_first = false;
                                    buf.write_str(&format_args(
                                        element.format,
                                        &element.args,
                                        hint,
                                    ))?;
                                }
                                buf.write_str("]")?;
                            }
                        }
                    }
                    Arg::Slice(x) => format_bytes(x, hint, &mut buf)?,
                    Arg::Char(c) => write!(buf, "{}", c)?,
                }
            }
        }
    }
    Ok(buf)
}
