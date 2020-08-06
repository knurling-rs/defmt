// NOTE: always runs on the host

use core::fmt::{self, Write as _};
use core::ops::Range;
use std::collections::BTreeMap;

use byteorder::{ReadBytesExt, LE};
use colored::Colorize;

use binfmt_parser::Type;
use common::Level;

/// Interner table that holds log levels and maps format strings to indices
#[derive(Debug)]
pub struct Table {
    pub entries: BTreeMap<usize, String>,
    pub debug: Range<usize>,
    pub error: Range<usize>,
    pub info: Range<usize>,
    pub trace: Range<usize>,
    pub warn: Range<usize>,
}

impl Table {
    // TODO constructor

    fn get(&self, index: usize) -> Result<(Option<Level>, &str), ()> {
        let lvl = if self.debug.contains(&index) {
            Some(Level::Debug)
        } else if self.error.contains(&index) {
            Some(Level::Error)
        } else if self.info.contains(&index) {
            Some(Level::Info)
        } else if self.trace.contains(&index) {
            Some(Level::Trace)
        } else if self.warn.contains(&index) {
            Some(Level::Warn)
        } else {
            None
        };

        Ok((lvl, &self.entries[&index]))
    }
}

/// A log frame
#[derive(Debug, PartialEq)]
pub struct Frame<'t> {
    pub level: Level,
    // Format string
    pub format: &'t str,
    pub timestamp: u64,
    pub args: Vec<Arg<'t>>,
}

impl<'t> Frame<'t> {
    pub fn display(&'t self, colored: bool) -> DisplayFrame<'t> {
        DisplayFrame {
            frame: self,
            colored,
        }
    }
}

pub struct DisplayFrame<'t> {
    frame: &'t Frame<'t>,
    colored: bool,
}

impl core::fmt::Display for DisplayFrame<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 0.000000 Info Hello, world!
        let seconds = self.frame.timestamp / 1000000;
        let micros = self.frame.timestamp % 1000000;

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

        let args = format_args(&self.frame.format, &self.frame.args);

        write!(f, "{}.{:06} {} {}", seconds, micros, level, args)
    }
}

// NOTE follows `parser::Type`
#[derive(Debug, PartialEq)]
pub enum Arg<'t> {
    /// Bool
    Bool(bool),
    F32(f32),
    /// U8, U16, U24 and U32
    Uxx(u64),
    /// I8, I16, I24 and I32
    Ixx(i64),
    /// Str
    Str(String),
    /// Interned string
    IStr(&'t str),
    /// Format
    Format {
        format: &'t str,
        args: Vec<Arg<'t>>,
    },
    /// Slice or Array of bytes.
    Slice(Vec<u8>),
}

impl fmt::Display for Arg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Arg::Bool(x) => write!(f, "{:?}", x),
            Arg::F32(x) => write!(f, "{}", ryu::Buffer::new().format(*x)),
            Arg::Uxx(x) => write!(f, "{}", x),
            Arg::Ixx(x) => write!(f, "{}", x),
            Arg::Str(x) => write!(f, "{}", x),
            Arg::IStr(x) => write!(f, "{}", x),
            Arg::Format { format, args } => f.write_str(&format_args(format, args)),
            Arg::Slice(x) => write!(f, "{:?}", x),
        }
    }
}

/// decode the data sent by the device using the previosuly stored metadata
///
/// * bytes: contains the data sent by the device that logs.
///          contains the [log string index, timestamp, optional fmt string args]
/// * table: contains the mapping of log string indices to their format strings, as well as the log level.
pub fn decode<'t>(
    mut bytes: &[u8],
    table: &'t Table,
) -> Result<(Frame<'t>, /*consumed: */ usize), ()> {
    let len = bytes.len();
    let index = leb128::read::unsigned(&mut bytes).map_err(drop)?;
    let timestamp = leb128::read::unsigned(&mut bytes).map_err(drop)?;

    let (level, format) = table.get(index as usize)?;
    let level = level.ok_or(())?;

    let args = parse_args(&mut bytes, format, table)?;

    let frame = Frame {
        level,
        format,
        timestamp,
        args,
    };

    let consumed = len - bytes.len();
    Ok((frame, consumed))
}

fn parse_args<'t>(bytes: &mut &[u8], format: &str, table: &'t Table) -> Result<Vec<Arg<'t>>, ()> {
    let mut args = vec![];
    let mut params = binfmt_parser::parse(format).map_err(drop)?;
    // sort & dedup to ensure that format string args can be addressed by index too
    params.sort_by_key(|param| param.index);
    params.dedup_by_key(|param| param.index);

    let mut bool_flags: u8 = 0; // the current group of consecutive bools
    let mut bools_left: u8 = 0; // the number of bits that we can still set in bool_flag

    for param in params {
        match param.ty {
            Type::U8 => {
                let data = bytes.read_u8().map_err(drop)?;
                args.push(Arg::Uxx(data as u64));
            }

            Type::Bool => {
                if bools_left == 0 {
                    bools_left = 8;
                    bool_flags = bytes.read_u8().map_err(drop)?;
                }
                // read out the lowest bit and turn it into a boolean
                let val = (bool_flags & 1) != 0;
                // drop the bit that we just read
                bool_flags >>= 1;
                bools_left -= 1;

                args.push(Arg::Bool(val));
            }

            Type::Format => {
                let index = leb128::read::unsigned(bytes).map_err(drop)?;
                let (level, format) = table.get(index as usize)?;
                // Auxiliary format strings must not have a log level associated with them.
                if level != None {
                    return Err(());
                }
                let inner_args = parse_args(bytes, format, table)?;

                args.push(Arg::Format {
                    format,
                    args: inner_args,
                });
            }
            Type::I16 => {
                let data = bytes.read_i16::<LE>().map_err(drop)?;
                args.push(Arg::Ixx(data as i64));
            }
            Type::I32 => {
                let data = bytes.read_i32::<LE>().map_err(drop)?;
                args.push(Arg::Ixx(data as i64));
            }
            Type::I8 => {
                let data = bytes.read_i8().map_err(drop)?;
                args.push(Arg::Ixx(data as i64));
            }
            Type::Isize => {
                // Signed isize is encoded in zigzag-encoding.
                let unsigned = leb128::read::unsigned(bytes).map_err(drop)?;
                args.push(Arg::Ixx(zigzag_decode(unsigned)))
            }
            Type::U16 => {
                let data = bytes.read_u16::<LE>().map_err(drop)?;
                args.push(Arg::Uxx(data as u64));
            }
            Type::U24 => {
                let data_low = bytes.read_u8().map_err(drop)?;
                let data_high = bytes.read_u16::<LE>().map_err(drop)?;
                let data = data_low as u64 | (data_high as u64) << 8;
                args.push(Arg::Uxx(data as u64));
            }
            Type::U32 => {
                let data = bytes.read_u32::<LE>().map_err(drop)?;
                args.push(Arg::Uxx(data as u64));
            }
            Type::Usize => {
                let unsigned = leb128::read::unsigned(bytes).map_err(drop)?;
                args.push(Arg::Uxx(unsigned))
            }
            Type::F32 => {
                let data = bytes.read_u32::<LE>().map_err(drop)?;
                args.push(Arg::F32(f32::from_bits(data)));
            }
            Type::BitField(_) => todo!(),
            Type::Str => {
                let str_len = leb128::read::unsigned(bytes).map_err(drop)? as usize;
                let mut arg_str_bytes = vec![];

                // note: went for the suboptimal but simple solution; optimize if necessary
                for _ in 0..str_len {
                    arg_str_bytes.push(bytes.read_u8().map_err(drop)?);
                }

                // convert to utf8 (no copy)
                let arg_str = String::from_utf8(arg_str_bytes).unwrap();

                args.push(Arg::Str(arg_str));
            }
            Type::IStr => {
                let str_index = leb128::read::unsigned(bytes).map_err(drop)? as usize;

                let (level, string) = table.get(str_index as usize)?;
                // Interned strings must not have a log level associated with them.
                if level != None {
                    return Err(());
                }

                args.push(Arg::IStr(string));
            }
            Type::Slice => {
                // only supports byte slices
                let num_elements = leb128::read::unsigned(bytes).map_err(drop)? as usize;
                let mut arg_slice = vec![];

                // note: went for the suboptimal but simple solution; optimize if necessary
                for _ in 0..num_elements {
                    arg_slice.push(bytes.read_u8().map_err(drop)?);
                }
                args.push(Arg::Slice(arg_slice.to_vec()));
            }
            Type::Array(len) => {
                let mut arg_slice = vec![];
                // note: went for the suboptimal but simple solution; optimize if necessary
                for _ in 0..len {
                    arg_slice.push(bytes.read_u8().map_err(drop)?);
                }
                args.push(Arg::Slice(arg_slice.to_vec()));
            }
        }
    }

    Ok(args)
}

fn format_args(format: &str, args: &[Arg]) -> String {
    fn unescape(literal: &str) -> String {
        literal.replace("{{", "{").replace("}}", "}")
    }

    let params = binfmt_parser::parse(format).unwrap();
    let mut buf = String::new();
    let mut cursor = 0;
    for param in params {
        let literal = &format[cursor..param.span.start];
        buf.push_str(&unescape(literal));
        cursor = param.span.end;

        write!(&mut buf, "{}", args[param.index]).ok();
    }
    buf.push_str(&unescape(&format[cursor..]));
    buf
}

fn zigzag_decode(unsigned: u64) -> i64 {
    (unsigned >> 1) as i64 ^ -((unsigned & 1) as i64)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use common::Level;

    use super::{Frame, Table};
    use crate::Arg;

    // helper function to initiate decoding and assert that the result is as expected.
    //
    // format:       format string to be expanded
    // bytes:        arguments + metadata
    // expectation:  the expected result
    fn decode_and_expect(format: &str, bytes: &[u8], expectation: &str) {
        let mut entries = BTreeMap::new();
        entries.insert(bytes[0] as usize, format.to_owned());

        let table = Table {
            entries,
            debug: 0..0,
            error: 0..0,
            info: 0..100, // enough space for many many args
            trace: 0..0,
            warn: 0..0,
        };

        let frame = super::decode(&bytes, &table).unwrap().0;
        assert_eq!(frame.display(false).to_string(), expectation.to_owned());
    }

    #[test]
    fn decode() {
        let mut entries = BTreeMap::new();
        entries.insert(0, "Hello, world!".to_owned());
        entries.insert(1, "The answer is {:u8}!".to_owned());
        // [IDX, TS, 42]
        //           ^^
        //entries.insert(2, "The answer is {0:u8} {1:u16}!".to_owned());

        let table = Table {
            entries,
            debug: 1..2,
            error: 0..0,
            info: 0..1,
            trace: 0..0,
            warn: 0..0,
        };

        let bytes = [0, 1];
        //     index ^  ^ timestamp

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Info,
                    format: "Hello, world!",
                    timestamp: 1,
                    args: vec![],
                },
                bytes.len(),
            ))
        );

        let bytes = [
            1,  // index
            2,  // timestamp
            42, // argument
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Debug,
                    format: "The answer is {:u8}!",
                    timestamp: 2,
                    args: vec![Arg::Uxx(42)],
                },
                bytes.len(),
            ))
        );

        // TODO Format ({:?})
    }

    #[test]
    fn all_integers() {
        const FMT: &str = "Hello, {:u8} {:u16} {:u24} {:u32} {:i8} {:i16} {:i32}!";
        let mut entries = BTreeMap::new();
        entries.insert(0, FMT.to_owned());

        let table = Table {
            entries,
            debug: 0..0,
            error: 0..0,
            info: 0..1,
            trace: 0..0,
            warn: 0..0,
        };

        let bytes = [
            0,  // index
            2,  // timestamp
            42, // u8
            0xff, 0xff, // u16
            0, 0, 1, // u24
            0xff, 0xff, 0xff, 0xff, // u32
            0xff, // i8
            0xff, 0xff, // i16
            0xff, 0xff, 0xff, 0xff, // i32
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Info,
                    format: FMT,
                    timestamp: 2,
                    args: vec![
                        Arg::Uxx(42),                      // u8
                        Arg::Uxx(u16::max_value().into()), // u16
                        Arg::Uxx(0x10000),                 // u24
                        Arg::Uxx(u32::max_value().into()), // u32
                        Arg::Ixx(-1),                      // i8
                        Arg::Ixx(-1),                      // i16
                        Arg::Ixx(-1),                      // i32
                    ],
                },
                bytes.len(),
            ))
        );
    }

    #[test]
    fn indices() {
        let mut entries = BTreeMap::new();
        entries.insert(0, "The answer is {0:u8} {0:u8}!".to_owned());
        entries.insert(1, "The answer is {1:u16} {0:u8} {1:u16}!".to_owned());

        let table = Table {
            entries,
            debug: 0..0,
            error: 0..0,
            info: 0..2,
            trace: 0..0,
            warn: 0..0,
        };
        let bytes = [
            0,  // index
            2,  // timestamp
            42, // argument
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Info,
                    format: "The answer is {0:u8} {0:u8}!",
                    timestamp: 2,
                    args: vec![Arg::Uxx(42)],
                },
                bytes.len(),
            ))
        );

        let bytes = [
            1,  // index
            2,  // timestamp
            42, // u8
            0xff, 0xff, // u16
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Info,
                    format: "The answer is {1:u16} {0:u8} {1:u16}!",
                    timestamp: 2,
                    args: vec![Arg::Uxx(42), Arg::Uxx(0xffff)],
                },
                bytes.len(),
            ))
        );
    }

    #[test]
    fn format() {
        let mut entries = BTreeMap::new();
        entries.insert(0, "x={:?}".to_owned());
        entries.insert(1, "Foo {{ x: {:u8} }}".to_owned());

        let table = Table {
            entries,
            debug: 0..0,
            error: 0..0,
            info: 0..1,
            trace: 0..0,
            warn: 0..0,
        };

        let bytes = [
            0,  // index
            2,  // timestamp
            1,  // index of the struct
            42, // Foo.x
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    level: Level::Info,
                    format: "x={:?}",
                    timestamp: 2,
                    args: vec![Arg::Format {
                        format: "Foo {{ x: {:u8} }}",
                        args: vec![Arg::Uxx(42)]
                    }],
                },
                bytes.len(),
            ))
        );
    }

    #[test]
    fn display() {
        let mut entries = BTreeMap::new();
        entries.insert(0, "x={:?}".to_owned());
        entries.insert(1, "Foo {{ x: {:u8} }}".to_owned());

        let table = Table {
            entries,
            debug: 0..0,
            error: 0..0,
            info: 0..1,
            trace: 0..0,
            warn: 0..0,
        };

        let bytes = [
            0,  // index
            2,  // timestamp
            1,  // index of the struct
            42, // Foo.x
        ];

        let frame = super::decode(&bytes, &table).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO x=Foo { x: 42 }"
        );
    }

    #[test]
    fn test_bools_simple() {
        let bytes = [
            0,          // index
            2,          // timestamp
            true as u8, // the logged bool value
        ];

        decode_and_expect("my bool={:bool}", &bytes, "0.000002 INFO my bool=true");
    }

    #[test]
    fn test_bools_more_than_fit_in_one_byte() {
        let bytes = [
            0,                // index
            2,                // timestamp
            0b10000110 as u8, // the first 8 logged bool values; in reverse order!
            0b1 as u8,        // the final logged bool value
        ];

        decode_and_expect(
            "bool overflow {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
            &bytes,
            "0.000002 INFO bool overflow false true true false false false false true true",
        );

        // Ensure that bools are compressed into the first byte even when there's non-bool values
        // between them.
        let bytes = [
            0,                // index
            2,                // timestamp
            0b10000110 as u8, // the first 8 logged bool values; in reverse order!
            0xff,             // a logged u8
            0b1 as u8,        // the final logged bool value
        ];

        decode_and_expect(
            "bool overflow {:bool} {:u8} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
            &bytes,
            "0.000002 INFO bool overflow false 255 true true false false false false true true",
        );
        decode_and_expect(
            "bool overflow {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:u8} {:bool}",
            &bytes,
            "0.000002 INFO bool overflow false true true false false false false true 255 true",
        );
    }

    #[test]
    fn test_bools_mixed() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b101 as u8, // 3 packed bools
            9 as u8,     // a uint in between
        ];

        decode_and_expect(
            "hidden bools {:bool} {:u8} {:bool} {:bool}",
            &bytes,
            "0.000002 INFO hidden bools true 9 false true",
        );
    }

    #[test]
    fn slice() {
        let bytes = [
            0, // index
            2, // timestamp
            2, // length of the slice
            23, 42, // slice content
        ];
        decode_and_expect("x={:[u8]}", &bytes, "0.000002 INFO x=[23, 42]");
    }

    #[test]
    fn slice_with_trailing_args() {
        let bytes = [
            0, // index
            2, // timestamp
            2, // length of the slice
            23, 42, // slice content
            1,  // trailing arg
        ];

        decode_and_expect(
            "x={:[u8]} trailing arg={:u8}",
            &bytes,
            "0.000002 INFO x=[23, 42] trailing arg=1",
        );
    }

    #[test]
    fn string_hello_world() {
        let bytes = [
            0, // index
            2, // timestamp
            5, // length of the string
            b'W', b'o', b'r', b'l', b'd',
        ];

        decode_and_expect("Hello {:str}", &bytes, "0.000002 INFO Hello World");
    }

    #[test]
    fn string_with_trailing_data() {
        let bytes = [
            0, // index
            2, // timestamp
            5, // length of the string
            b'W', b'o', b'r', b'l', b'd', 125, // trailing data
        ];

        decode_and_expect(
            "Hello {:str} {:u8}",
            &bytes,
            "0.000002 INFO Hello World 125",
        );
    }
}
