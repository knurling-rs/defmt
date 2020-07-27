// NOTE: always runs on the host

use core::fmt::{self, Write as _};
use core::ops::Range;
use std::collections::BTreeMap;

use byteorder::{ReadBytesExt, LE};
use colored::Colorize;

use binfmt_parser::Type;
use common::Level;

/// Interner table
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

impl core::fmt::Display for Frame<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 0.000000 Info Hello, world!
        let seconds = self.timestamp / 1000000;
        let micros = self.timestamp % 1000000;

        let level = match self.level {
            Level::Trace => "TRACE".dimmed(),
            Level::Debug => "DEBUG".normal(),
            Level::Info => "INFO".green(),
            Level::Warn => "WARN".yellow(),
            Level::Error => "ERROR".red(),
        };

        let params = binfmt_parser::parse(self.format).unwrap();
        let mut buf = String::new();
        let mut cursor = 0;
        for param in params {
            //let tocopy = param.span.start - cursor;
            buf.push_str(&self.format[cursor..param.span.start]);
            cursor = param.span.end;

            write!(&mut buf, "{}", self.args[param.index]).ok();
        }

        write!(f, "{}.{:06} {}", seconds, micros, level)
    }
}

// NOTE follows `parser::Type`
#[derive(Debug, PartialEq)]
pub enum Arg<'t> {
    // Bool
    Bool(bool),
    // U8, U16, U24 and U32
    Uxx(u64),
    // I8, I16, I24 and I32
    Ixx(i64),
    // Str
    Str(&'t str),
    // Format
    Format { format: &'t str, args: Vec<Arg<'t>> },
    // Slice
    Slice(Vec<u8>),
}

impl fmt::Display for Arg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!()
    }
}

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
    params.sort_by_key(|param| param.index);
    params.dedup_by_key(|param| param.index);

    for param in params {
        match param.ty {
            Type::U8 => {
                let data = bytes.read_u8().map_err(drop)?;
                args.push(Arg::Uxx(data as u64));
            }

            Type::BitField(_) => {}
            Type::Bool => {}
            // {:?}
            Type::Format => {
                let index = leb128::read::unsigned(bytes).map_err(drop)?;
                let (level, format) = table.get(index as usize)?;
                // not well-formed
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
            Type::Str => {}
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
            Type::Slice => {}
        }
    }

    Ok(args)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use common::Level;

    use super::{Frame, Table};
    use crate::Arg;

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
}
