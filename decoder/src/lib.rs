// NOTE: always runs on the host

use core::ops::Range;
use std::collections::BTreeMap;

use common::Level;

/// Interner table
pub struct Table {
    entries: BTreeMap<usize, String>,
    debug: Range<usize>,
    error: Range<usize>,
    info: Range<usize>,
    trace: Range<usize>,
    warn: Range<usize>,
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

pub fn decode<'t>(
    mut bytes: &[u8],
    table: &'t Table,
) -> Result<(Frame<'t>, /*consumed: */ usize), ()> {
    let len = bytes.len();
    let index = leb128::read::unsigned(&mut bytes).map_err(drop)?;
    let timestamp = leb128::read::unsigned(&mut bytes).map_err(drop)?;

    let (level, format) = table.get(index as usize)?;
    let level = level.ok_or(())?;

    let consumed = len - bytes.len();
    let frame = Frame {
        level,
        format,
        timestamp,
        args: Vec::new(),
    };

    Ok((frame, consumed))
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
                2
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
                    level: Level::Info,
                    format: "The answer is {:u8}!",
                    timestamp: 2,
                    args: vec![Arg::Uxx(42)],
                },
                2
            ))
        );

        // TODO Format ({:?})
    }
}
