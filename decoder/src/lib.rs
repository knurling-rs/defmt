//! Decodes [`defmt`](https://github.com/knurling-rs/defmt) log frames
//!
//! NOTE: The decoder always runs on the host!
//!
//! This is an implementation detail of [`probe-run`](https://github.com/knurling-rs/probe-run) and
//! not meant to be consumed by other tools at the moment so all the API is unstable.

#![cfg(feature = "unstable")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(cfg(unstable)))]
#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

// load DEFMT_VERSION
include!(concat!(env!("OUT_DIR"), "/version.rs"));

mod decoder;
mod elf2table;
mod frame;
pub mod log;

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, io,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use decoder::{read_leb128, Decoder};
use defmt_parser::Level;
use elf2table::parse_impl;
use frame::Frame;

pub use elf2table::{Location, Locations};

#[derive(PartialEq, Eq, Debug)]
pub enum Tag {
    /// Defmt-controlled format string for primitive types.
    Prim,
    /// Format string created by `#[derive(Format)]`.
    Derived,
    /// A user-defined format string from a `write!` invocation.
    Write,
    /// An interned string, for use with `{=istr}`.
    Str,
    /// Defines the global timestamp format.
    Timestamp,

    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Tag {
    fn to_level(&self) -> Option<Level> {
        match self {
            Tag::Trace => Some(Level::Trace),
            Tag::Debug => Some(Level::Debug),
            Tag::Info => Some(Level::Info),
            Tag::Warn => Some(Level::Warn),
            Tag::Error => Some(Level::Error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct TableEntry {
    string: StringEntry,
    raw_symbol: String,
}

impl TableEntry {
    pub fn new(string: StringEntry, raw_symbol: String) -> Self {
        Self { string, raw_symbol }
    }

    #[cfg(test)]
    fn new_without_symbol(tag: Tag, string: String) -> Self {
        Self {
            string: StringEntry::new(tag, string),
            raw_symbol: "<unknown>".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct StringEntry {
    tag: Tag,
    string: String,
}

impl StringEntry {
    pub fn new(tag: Tag, string: String) -> Self {
        Self { tag, string }
    }
}

/// Internal table that holds log levels and maps format strings to indices
#[derive(Debug)]
pub struct Table {
    timestamp: Option<TableEntry>,
    entries: BTreeMap<usize, TableEntry>,
}

impl Table {
    /// NOTE caller must verify that defmt symbols are compatible with this version of the `decoder` crate using the `check_version` function
    pub fn new(entries: BTreeMap<usize, TableEntry>) -> Self {
        Self {
            entries,
            timestamp: None,
        }
    }

    /// Parses an ELF file and returns the decoded `defmt` table.
    ///
    /// This function returns `None` if the ELF file contains no `.defmt` section.
    pub fn parse(elf: &[u8]) -> Result<Option<Table>, anyhow::Error> {
        parse_impl(elf, true)
    }

    /// Like `parse`, but does not verify that the defmt version in the firmware matches the host.
    ///
    /// CAUTION: This is meant for defmt/probe-run development only and can result in reading garbage data.
    pub fn parse_ignore_version(elf: &[u8]) -> Result<Option<Table>, anyhow::Error> {
        parse_impl(elf, false)
    }

    pub fn set_timestamp_entry(&mut self, timestamp: TableEntry) {
        self.timestamp = Some(timestamp);
    }

    fn _get(&self, index: usize) -> Result<(Option<Level>, &str), ()> {
        let entry = self.entries.get(&index).ok_or(())?;
        Ok((entry.string.tag.to_level(), &entry.string.string))
    }

    fn get_with_level(&self, index: usize) -> Result<(Level, &str), ()> {
        let (lvl, format) = self._get(index)?;
        Ok((lvl.ok_or(())?, format))
    }

    fn get_without_level(&self, index: usize) -> Result<&str, ()> {
        let (lvl, format) = self._get(index)?;
        if lvl.is_none() {
            Ok(format)
        } else {
            Err(())
        }
    }

    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.entries.iter().filter_map(move |(idx, entry)| {
            if entry.string.tag.to_level().is_some() {
                Some(*idx)
            } else {
                None
            }
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over the raw symbols of the table entries
    pub fn raw_symbols(&self) -> impl Iterator<Item = &str> + '_ {
        self.entries.values().map(|s| &*s.raw_symbol)
    }

    pub fn get_locations(&self, elf: &[u8]) -> Result<Locations, anyhow::Error> {
        elf2table::get_locations(elf, self)
    }

    /// decode the data sent by the device using the previosuly stored metadata
    ///
    /// * bytes: contains the data sent by the device that logs.
    ///          contains the [log string index, timestamp, optional fmt string args]
    pub fn decode<'t>(
        &'t self,
        mut bytes: &[u8],
    ) -> Result<(Frame<'t>, /*consumed: */ usize), DecodeError> {
        let len = bytes.len();
        let index = read_leb128(&mut bytes)?;

        let mut decoder = Decoder::new(self, bytes);

        let mut timestamp_format = None;
        let mut timestamp_args = Vec::new();
        if let Some(entry) = self.timestamp.as_ref() {
            let format = &entry.string.string;
            timestamp_format = Some(&**format);
            timestamp_args = decoder.decode_format(format)?;
        }

        let (level, format) = self
            .get_with_level(index as usize)
            .map_err(|_| DecodeError::Malformed)?;

        let args = decoder.decode_format(format)?;
        if !decoder.bools_tbd.is_empty() {
            // Flush end of compression block.
            decoder.read_and_unpack_bools()?;
        }

        let frame = Frame::new(level, index, timestamp_format, timestamp_args, format, args);

        let consumed = len - decoder.bytes.len();
        Ok((frame, consumed))
    }
}

#[derive(Debug)]
struct Bool(AtomicBool);

impl Bool {
    #[allow(clippy::declare_interior_mutable_const)]
    const FALSE: Self = Self(AtomicBool::new(false));

    fn set(&self, value: bool) {
        self.0.store(value, atomic::Ordering::Relaxed);
    }
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0.load(atomic::Ordering::Relaxed))
    }
}

impl PartialEq for Bool {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .load(atomic::Ordering::Relaxed)
            .eq(&other.0.load(atomic::Ordering::Relaxed))
    }
}

// NOTE follows `parser::Type`
#[derive(Debug, PartialEq)]
enum Arg<'t> {
    /// Bool
    Bool(Arc<Bool>),
    F32(f32),
    F64(f64),
    /// U8, U16, U24 and U32
    Uxx(u128),
    /// I8, I16, I24 and I32
    Ixx(i128),
    /// Str
    Str(String),
    /// Interned string
    IStr(&'t str),
    /// Format
    Format {
        format: &'t str,
        args: Vec<Arg<'t>>,
    },
    FormatSlice {
        elements: Vec<FormatSliceElement<'t>>,
    },
    /// Slice or Array of bytes.
    Slice(Vec<u8>),
    /// Char
    Char(char),

    /// `fmt::Debug` / `fmt::Display` formatted on-target.
    Preformatted(String),
}

#[derive(Debug, PartialEq)]
struct FormatSliceElement<'t> {
    // this will usually be the same format string for all elements; except when the format string
    // is an enum -- in that case `format` will be the variant
    format: &'t str,
    args: Vec<Arg<'t>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// More data is needed to decode the next frame.
    UnexpectedEof,

    Malformed,
}

impl From<io::Error> for DecodeError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            DecodeError::UnexpectedEof
        } else {
            DecodeError::Malformed
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnexpectedEof => f.write_str("unexpected end of stream"),
            DecodeError::Malformed => f.write_str("malformed data"),
        }
    }
}

impl Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // helper function to initiate decoding and assert that the result is as expected.
    //
    // format:       format string to be expanded
    // bytes:        arguments + metadata
    // expectation:  the expected result
    fn decode_and_expect(format: &str, bytes: &[u8], expectation: &str) {
        let mut entries = BTreeMap::new();
        entries.insert(
            bytes[0] as usize,
            TableEntry::new_without_symbol(Tag::Info, format.to_string()),
        );

        let table = Table {
            entries,
            timestamp: Some(TableEntry::new_without_symbol(
                Tag::Timestamp,
                "{=u8:µs}".to_owned(),
            )),
        };

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), expectation.to_owned());
    }

    #[test]
    fn decode() {
        let mut entries = BTreeMap::new();
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "Hello, world!".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(Tag::Debug, "The answer is {=u8}!".to_owned()),
        );
        // [IDX, TS, 42]
        //           ^^
        //entries.insert(2, "The answer is {0:u8} {1:u16}!".to_owned());

        let table = Table {
            entries,
            timestamp: None,
        };

        let bytes = [0];
        //     index ^

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(Level::Info, 0, None, vec![], "Hello, world!", vec![],),
                bytes.len(),
            ))
        );

        let bytes = [
            1,  // index
            42, // argument
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    Level::Debug,
                    1,
                    None,
                    vec![],
                    "The answer is {=u8}!",
                    vec![Arg::Uxx(42)],
                ),
                bytes.len(),
            ))
        );

        // TODO Format ({:?})
    }

    #[test]
    fn all_integers() {
        const FMT: &str =
            "Hello, {=u8} {=u16} {=u24} {=u32} {=u64} {=u128} {=i8} {=i16} {=i32} {=i64} {=i128}!";
        let mut entries = BTreeMap::new();
        entries.insert(0, TableEntry::new_without_symbol(Tag::Info, FMT.to_owned()));

        let table = Table {
            entries,
            timestamp: None,
        };

        let bytes = [
            0,  // index
            42, // u8
            0xff, 0xff, // u16
            0, 0, 1, // u24
            0xff, 0xff, 0xff, 0xff, // u32
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // u64
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, // u128
            0xff, // i8
            0xff, 0xff, // i16
            0xff, 0xff, 0xff, 0xff, // i32
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // i64
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, // i128
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    Level::Info,
                    0,
                    None,
                    vec![],
                    FMT,
                    vec![
                        Arg::Uxx(42),                      // u8
                        Arg::Uxx(u16::max_value().into()), // u16
                        Arg::Uxx(0x10000),                 // u24
                        Arg::Uxx(u32::max_value().into()), // u32
                        Arg::Uxx(u64::max_value().into()), // u64
                        Arg::Uxx(u128::max_value()),       // u128
                        Arg::Ixx(-1),                      // i8
                        Arg::Ixx(-1),                      // i16
                        Arg::Ixx(-1),                      // i32
                        Arg::Ixx(-1),                      // i64
                        Arg::Ixx(-1),                      // i128
                    ],
                ),
                bytes.len(),
            ))
        );
    }

    #[test]
    fn indices() {
        let mut entries = BTreeMap::new();
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "The answer is {0=u8} {0=u8}!".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(
                Tag::Info,
                "The answer is {1=u16} {0=u8} {1=u16}!".to_owned(),
            ),
        );

        let table = Table {
            entries,
            timestamp: None,
        };
        let bytes = [
            0,  // index
            42, // argument
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    Level::Info,
                    0,
                    None,
                    vec![],
                    "The answer is {0=u8} {0=u8}!",
                    vec![Arg::Uxx(42)],
                ),
                bytes.len(),
            ))
        );

        let bytes = [
            1,  // index
            42, // u8
            0xff, 0xff, // u16
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    Level::Info,
                    1,
                    None,
                    vec![],
                    "The answer is {1=u16} {0=u8} {1=u16}!",
                    vec![Arg::Uxx(42), Arg::Uxx(0xffff)],
                ),
                bytes.len(),
            ))
        );
    }

    #[test]
    fn format() {
        let mut entries = BTreeMap::new();
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "x={=?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(Tag::Derived, "Foo {{ x: {=u8} }}".to_owned()),
        );

        let table = Table {
            entries,
            timestamp: None,
        };

        let bytes = [
            0,  // index
            1,  // index of the struct
            42, // Foo.x
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    Level::Info,
                    0,
                    None,
                    vec![],
                    "x={=?}",
                    vec![Arg::Format {
                        format: "Foo {{ x: {=u8} }}",
                        args: vec![Arg::Uxx(42)]
                    }],
                ),
                bytes.len(),
            ))
        );
    }

    #[test]
    fn display() {
        let mut entries = BTreeMap::new();
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "x={=?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(Tag::Derived, "Foo {{ x: {=u8} }}".to_owned()),
        );

        let table = Table {
            entries,
            timestamp: Some(TableEntry::new_without_symbol(
                Tag::Timestamp,
                "{=u8:µs}".to_owned(),
            )),
        };

        let bytes = [
            0,  // index
            2,  // timestamp
            1,  // index of the struct
            42, // Foo.x
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO x=Foo { x: 42 }"
        );
    }

    #[test]
    fn bools_simple() {
        let bytes = [
            0,          // index
            2,          // timestamp
            true as u8, // the logged bool value
        ];

        decode_and_expect("my bool={=bool}", &bytes, "0.000002 INFO my bool=true");
    }

    #[test]
    fn bools_max_capacity() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0001, // the first 8 logged bool values
        ];

        decode_and_expect(
            "bool capacity {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool}",
            &bytes,
            "0.000002 INFO bool capacity false true true false false false false true",
        );
    }

    #[test]
    fn bools_more_than_fit_in_one_byte() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0001, // the first 8 logged bool values
            0b1,         // the final logged bool value
        ];

        decode_and_expect(
            "bool overflow {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool}",
            &bytes,
            "0.000002 INFO bool overflow false true true false false false false true true",
        );

        // Ensure that bools are compressed into the first byte even when there's non-bool values
        // between them.
        let bytes = [
            0,           // index
            2,           // timestamp
            0xff,        // the logged u8
            0b0110_0001, // the first 8 logged bool values
            0b1,         // the final logged bool value
        ];

        decode_and_expect(
            "bool overflow {=bool} {=u8} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool}",
            &bytes,
            "0.000002 INFO bool overflow false 255 true true false false false false true true",
        );

        // Ensure that bools are compressed into the first byte even when there's a non-bool value
        // right between between the two compression blocks.
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0001, // the first 8 logged bool values
            0xff,        // the logged u8
            0b1,         // the final logged bool value
        ];

        decode_and_expect(
            "bool overflow {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=bool} {=u8} {=bool}",
            &bytes,
            "0.000002 INFO bool overflow false true true false false false false true 255 true",
        );
    }

    #[test]
    fn bools_mixed() {
        let bytes = [
            0,       // index
            2,       // timestamp
            9 as u8, // a uint in between
            0b101,   // 3 packed bools
        ];

        decode_and_expect(
            "hidden bools {=bool} {=u8} {=bool} {=bool}",
            &bytes,
            "0.000002 INFO hidden bools true 9 false true",
        );
    }

    #[test]
    fn bools_mixed_no_trailing_bool() {
        let bytes = [
            0,   // index
            2,   // timestamp
            9,   // a u8 in between
            0b0, // 3 packed bools
        ];

        decode_and_expect(
            "no trailing bools {=bool} {=u8}",
            &bytes,
            "0.000002 INFO no trailing bools false 9",
        );
    }

    #[test]
    fn bools_bool_struct() {
        /*
        emulate
        #[derive(Format)]
        struct Flags {
            a: bool,
            b: bool,
            c: bool,
        }

        defmt::info!("{:bool} {:?}", true, Flags {a: true, b: false, c: true });
        */

        let mut entries = BTreeMap::new();
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "{=bool} {=?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(
                Tag::Derived,
                "Flags {{ a: {=bool}, b: {=bool}, c: {=bool} }}".to_owned(),
            ),
        );

        let table = Table {
            entries,
            timestamp: Some(TableEntry::new_without_symbol(
                Tag::Timestamp,
                "{=u8:µs}".to_owned(),
            )),
        };

        let bytes = [
            0,      // index
            2,      // timestamp
            1,      // index of Flags { a: {:bool}, b: {:bool}, c: {:bool} }
            0b1101, // 4 packed bools
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO true Flags { a: true, b: false, c: true }"
        );
    }

    #[test]
    fn bitfields() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b1110_0101, // u8
        ];
        decode_and_expect(
            "x: {0=0..4:b}, y: {0=3..8:b}",
            &bytes,
            "0.000002 INFO x: 0b101, y: 0b11100",
        );
    }

    #[test]
    fn bitfields_reverse_order() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b1101_0010, // u8
        ];
        decode_and_expect(
            "x: {0=0..7:b}, y: {0=3..5:b}",
            &bytes,
            "0.000002 INFO x: 0b1010010, y: 0b10",
        );
    }

    #[test]
    fn bitfields_different_indices() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b1111_0000, // u8
            0b1110_0101, // u8
        ];
        decode_and_expect(
            "#0: {0=0..5:b}, #1: {1=3..8:b}",
            &bytes,
            "0.000002 INFO #0: 0b10000, #1: 0b11100",
        );
    }

    #[test]
    fn bitfields_u16() {
        let bytes = [
            0, // index
            2, // timestamp
            0b1111_0000,
            0b1110_0101, // u16
        ];
        decode_and_expect("x: {0=7..12:b}", &bytes, "0.000002 INFO x: 0b1011");
    }

    #[test]
    fn bitfields_mixed_types() {
        let bytes = [
            0, // index
            2, // timestamp
            0b1111_0000,
            0b1110_0101, // u16
            0b1111_0001, // u8
        ];
        decode_and_expect(
            "#0: {0=7..12:b}, #1: {1=0..5:b}",
            &bytes,
            "0.000002 INFO #0: 0b1011, #1: 0b10001",
        );
    }

    #[test]
    fn bitfields_mixed() {
        let bytes = [
            0, // index
            2, // timestamp
            0b1111_0000,
            0b1110_0101, // u16 bitfields
            42,          // u8
            0b1111_0001, // u8 bitfields
        ];
        decode_and_expect(
            "#0: {0=7..12:b}, #1: {1=u8}, #2: {2=0..5:b}",
            &bytes,
            "0.000002 INFO #0: 0b1011, #1: 42, #2: 0b10001",
        );
    }

    #[test]
    fn bitfields_across_boundaries() {
        let bytes = [
            0, // index
            2, // timestamp
            0b1101_0010,
            0b0110_0011, // u16
        ];
        decode_and_expect(
            "bitfields {0=0..7:b} {0=9..14:b}",
            &bytes,
            "0.000002 INFO bitfields 0b1010010 0b10001",
        );
    }

    #[test]
    fn bitfields_across_boundaries_diff_indices() {
        let bytes = [
            0, // index
            2, // timestamp
            0b1101_0010,
            0b0110_0011, // u16
            0b1111_1111, // truncated u16
        ];
        decode_and_expect(
            "bitfields {0=0..7:b} {0=9..14:b} {1=8..10:b}",
            &bytes,
            "0.000002 INFO bitfields 0b1010010 0b10001 0b11",
        );
    }

    #[test]
    fn bitfields_truncated_front() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0011, // truncated(!) u16
        ];
        decode_and_expect(
            "bitfields {0=9..14:b}",
            &bytes,
            "0.000002 INFO bitfields 0b10001",
        );
    }

    #[test]
    fn bitfields_non_truncated_u32() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0011, // -
            0b0000_1111, //  |
            0b0101_1010, //  | u32
            0b1100_0011, // -
        ];
        decode_and_expect(
            "bitfields {0=0..2:b} {0=28..31:b}",
            &bytes,
            "0.000002 INFO bitfields 0b11 0b100",
        );
    }

    #[test]
    fn bitfields_u128() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b1110_0101, // 120..127
            0b1110_0101, // 112..119
            0b0000_0000, // 104..111
            0b0000_0000, // 96..103
            0b0000_0000, // 88..95
            0b0000_0000, // 80..87
            0b0000_0000, // 72..79
            0b0000_0000, // 64..71
            0b0000_0000, // 56..63
            0b0000_0000, // 48..55
            0b0000_0000, // 40..47
            0b0000_0000, // 32..39
            0b0000_0000, // 24..31
            0b0000_0000, // 16..23
            0b0000_0000, // 8..15
            0b0000_0000, // 0..7
        ];
        decode_and_expect("x: {0=119..124:b}", &bytes, "0.000002 INFO x: 0b1011");
    }

    #[test]
    fn slice() {
        let bytes = [
            0, // index
            2, // timestamp
            2, // length of the slice
            23, 42, // slice content
        ];
        decode_and_expect("x={=[u8]}", &bytes, "0.000002 INFO x=[23, 42]");
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
            "x={=[u8]} trailing arg={=u8}",
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

        decode_and_expect("Hello {=str}", &bytes, "0.000002 INFO Hello World");
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
            "Hello {=str} {=u8}",
            &bytes,
            "0.000002 INFO Hello World 125",
        );
    }

    #[test]
    fn char_data() {
        let bytes = [
            0, // index
            2, // timestamp
            0x61, 0x00, 0x00, 0x00, // char 'a'
            0x9C, 0xF4, 0x01, 0x00, // Purple heart emoji
        ];

        decode_and_expect(
            "Supports ASCII {=char} and Unicode {=char}",
            &bytes,
            "0.000002 INFO Supports ASCII a and Unicode 💜",
        );
    }

    #[test]
    fn option() {
        let mut entries = BTreeMap::new();
        entries.insert(
            4,
            TableEntry::new_without_symbol(Tag::Info, "x={=?}".to_owned()),
        );
        entries.insert(
            3,
            TableEntry::new_without_symbol(Tag::Derived, "None|Some({=?})".to_owned()),
        );
        entries.insert(
            2,
            TableEntry::new_without_symbol(Tag::Derived, "{=u8}".to_owned()),
        );

        let table = Table {
            entries,
            timestamp: Some(TableEntry::new_without_symbol(
                Tag::Timestamp,
                "{=u8:µs}".to_owned(),
            )),
        };

        let bytes = [
            4,  // string index (INFO)
            0,  // timestamp
            3,  // string index (enum)
            1,  // Some discriminant
            2,  // string index (u8)
            42, // Some.0
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000000 INFO x=Some(42)");

        let bytes = [
            4, // string index (INFO)
            1, // timestamp
            3, // string index (enum)
            0, // None discriminant
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000001 INFO x=None");
    }
}
