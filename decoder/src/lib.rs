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

pub const DEFMT_VERSION: &str = "3";

mod decoder;
mod elf2table;
mod frame;
pub mod log;
mod stream;

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt, io,
    str::FromStr,
};

use byteorder::{ReadBytesExt, LE};
use decoder::Decoder;
use defmt_parser::Level;
use elf2table::parse_impl;

pub use elf2table::{Location, Locations};
pub use frame::Frame;
pub use stream::StreamDecoder;

/// Specifies the origin of a format string
#[derive(PartialEq, Eq, Debug)]
pub enum Tag {
    /// Defmt-controlled format string for primitive types.
    Prim,
    /// Format string created by `#[derive(Format)]`.
    Derived,
    /// Format string created by `defmt::bitflags!`.
    Bitflags,
    /// A user-defined format string from a `write!` invocation.
    Write,
    /// An interned string, for use with `{=istr}`.
    Str,
    /// Defines the global timestamp format.
    Timestamp,

    /// `static` containing a possible value of a bitflags type.
    BitflagsValue,
    /// Format string created by `defmt::println!`.
    Println,

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

/// Entry in [`Table`] combining a format string with its raw symbol
#[derive(Debug, PartialEq)]
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

/// A format string and it's [`Tag`]
#[derive(Debug, PartialEq)]
pub struct StringEntry {
    tag: Tag,
    string: String,
}

impl StringEntry {
    pub fn new(tag: Tag, string: String) -> Self {
        Self { tag, string }
    }
}

/// Data that uniquely identifies a `defmt::bitflags!` invocation.
#[derive(Debug, PartialEq, Eq, Hash)]
struct BitflagsKey {
    /// Name of the bitflags struct (this is really redundant with `disambig`).
    ident: String,
    package: String,
    disambig: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Encoding {
    Raw,
    Rzcobs,
}

impl FromStr for Encoding {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "raw" => Ok(Encoding::Raw),
            "rzcobs" => Ok(Encoding::Rzcobs),
            _ => anyhow::bail!("Unknown defmt encoding '{}' specified. This is a bug.", s),
        }
    }
}

impl Encoding {
    pub const fn can_recover(&self) -> bool {
        match self {
            Encoding::Raw => false,
            Encoding::Rzcobs => true,
        }
    }
}

/// Internal table that holds log levels and maps format strings to indices
#[derive(Debug, PartialEq)]
pub struct Table {
    timestamp: Option<TableEntry>,
    entries: BTreeMap<usize, TableEntry>,
    bitflags: HashMap<BitflagsKey, Vec<(String, u128)>>,
    encoding: Encoding,
}

impl Table {
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

    fn get_with_level(&self, index: usize) -> Result<(Option<Level>, &str), ()> {
        self._get(index)
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
            if entry.string.tag.to_level().is_some() || entry.string.tag == Tag::Println {
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

    /// Decode the data sent by the device using the previously stored metadata.
    ///
    /// * `bytes`
    ///   * contains the data sent by the device that logs.
    ///   * contains the [log string index, timestamp, optional fmt string args]
    pub fn decode<'t>(
        &'t self,
        mut bytes: &[u8],
    ) -> Result<(Frame<'t>, /* consumed: */ usize), DecodeError> {
        let len = bytes.len();
        let index = bytes.read_u16::<LE>()? as u64;

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

        let frame = Frame::new(
            self,
            level,
            index,
            timestamp_format,
            timestamp_args,
            format,
            args,
        );

        let consumed = len - decoder.bytes.len();
        Ok((frame, consumed))
    }

    pub fn new_stream_decoder(&self) -> Box<dyn StreamDecoder + '_> {
        match self.encoding {
            Encoding::Raw => Box::new(stream::Raw::new(self)),
            Encoding::Rzcobs => Box::new(stream::Rzcobs::new(self)),
        }
    }

    pub fn encoding(&self) -> Encoding {
        self.encoding
    }
}

// NOTE follows `parser::Type`
#[derive(Debug, Clone, PartialEq)]
enum Arg<'t> {
    /// Bool
    Bool(bool),
    F32(f32),
    F64(f64),
    /// U8, U16, U32, U64, U128
    Uxx(u128),
    /// I8, I16, I32, I64, I128
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
    FormatSequence {
        args: Vec<Arg<'t>>,
    },
    /// Slice or Array of bytes.
    Slice(Vec<u8>),
    /// Char
    Char(char),

    /// `fmt::Debug` / `fmt::Display` formatted on-target.
    Preformatted(String),
}

#[derive(Debug, Clone, PartialEq)]
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

    fn test_table(entries: impl IntoIterator<Item = TableEntry>) -> Table {
        Table {
            timestamp: None,
            entries: entries.into_iter().enumerate().collect(),
            bitflags: Default::default(),
            encoding: Encoding::Raw,
        }
    }

    fn test_table_with_timestamp(
        entries: impl IntoIterator<Item = TableEntry>,
        timestamp: &str,
    ) -> Table {
        Table {
            timestamp: Some(TableEntry::new_without_symbol(
                Tag::Timestamp,
                timestamp.into(),
            )),
            entries: entries.into_iter().enumerate().collect(),
            bitflags: Default::default(),
            encoding: Encoding::Raw,
        }
    }

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
                "{=u8:us}".to_owned(),
            )),
            bitflags: Default::default(),
            encoding: Encoding::Raw,
        };

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), expectation.to_owned());
    }

    #[test]
    fn decode() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "Hello, world!".to_owned()),
            TableEntry::new_without_symbol(Tag::Debug, "The answer is {=u8}!".to_owned()),
        ];

        let table = test_table(entries);

        let bytes = [0, 0];
        //     index ^

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Info),
                    0,
                    None,
                    vec![],
                    "Hello, world!",
                    vec![],
                ),
                bytes.len(),
            ))
        );

        let bytes = [
            1, 0,  // index
            42, // argument
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Debug),
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
            "Hello, {=u8} {=u16} {=u32} {=u64} {=u128} {=i8} {=i16} {=i32} {=i64} {=i128}!";

        let entries = vec![TableEntry::new_without_symbol(Tag::Info, FMT.to_owned())];

        let table = test_table(entries);

        let bytes = [
            0, 0,  // index
            42, // u8
            0xff, 0xff, // u16
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
                    &table,
                    Some(Level::Info),
                    0,
                    None,
                    vec![],
                    FMT,
                    vec![
                        Arg::Uxx(42),                      // u8
                        Arg::Uxx(u16::max_value().into()), // u16
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
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "The answer is {0=u8} {0=u8}!".to_owned()),
            TableEntry::new_without_symbol(
                Tag::Info,
                "The answer is {1=u16} {0=u8} {1=u16}!".to_owned(),
            ),
        ];

        let table = test_table(entries);
        let bytes = [
            0, 0,  // index
            42, // argument
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Info),
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
            1, 0,  // index
            42, // u8
            0xff, 0xff, // u16
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Info),
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
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "x={=?}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "Foo {{ x: {=u8} }}".to_owned()),
        ];

        let table = test_table(entries);

        let bytes = [
            0, 0, // index
            1, 0,  // index of the struct
            42, // Foo.x
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Info),
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
    fn format_sequence() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "{=__internal_FormatSequence}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "Foo".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "Bar({=u8})".to_owned()),
        ];

        let table = test_table(entries);

        let bytes = [
            0, 0, // index
            1, 0, // index of Foo
            2, 0,  // index of Bar
            42, // bar.x
            0, 0, // terminator
        ];

        assert_eq!(
            table.decode(&bytes),
            Ok((
                Frame::new(
                    &table,
                    Some(Level::Info),
                    0,
                    None,
                    vec![],
                    "{=__internal_FormatSequence}",
                    vec![Arg::FormatSequence {
                        args: vec![
                            Arg::Format {
                                format: "Foo",
                                args: vec![]
                            },
                            Arg::Format {
                                format: "Bar({=u8})",
                                args: vec![Arg::Uxx(42)]
                            }
                        ]
                    }],
                ),
                bytes.len(),
            ))
        );
    }

    #[test]
    fn display() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "x={=?}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "Foo {{ x: {=u8} }}".to_owned()),
        ];

        let table = test_table_with_timestamp(entries, "{=u8:us}");

        let bytes = [
            0, 0, // index
            2, // timestamp
            1, 0,  // index of the struct
            42, // Foo.x
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO x=Foo { x: 42 }"
        );
    }

    #[test]
    fn display_use_inner_type_hint() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "x={:b}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "S {{ x: {=u8:x} }}".to_owned()),
        ];

        let table = test_table_with_timestamp(entries, "{=u8:us}");

        let bytes = [
            0, 0, // index
            2, // timestamp
            1, 0,  // index of the struct
            42, // value
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO x=S { x: 2a }",
        );
    }

    #[test]
    fn display_use_outer_type_hint() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "x={:b}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "S {{ x: {=u8:?} }}".to_owned()),
        ];

        let table = test_table_with_timestamp(entries, "{=u8:us}");

        let bytes = [
            0, 0, // index
            2, // timestamp
            1, 0,  // index of the struct
            42, // value
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO x=S { x: 101010 }",
        );
    }

    #[test]
    fn display_inner_str_in_struct() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Info, "{}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "S {{ x: {=str:?} }}".to_owned()),
        ];

        let table = test_table_with_timestamp(entries, "{=u8:us}");

        let bytes = [
            0, 0, // index
            2, // timestamp
            1, 0, // index into the struct
            5, 0, 0, 0, // length of the string
            b'H', b'e', b'l', b'l', b'o', // string "Hello"
        ];
        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO S { x: \"Hello\" }",
        );
    }

    #[test]
    fn display_u8_vec() {
        let entries = vec![
            TableEntry::new_without_symbol(Tag::Prim, "{=u8}".to_owned()),
            TableEntry::new_without_symbol(Tag::Prim, "{=[?]}".to_owned()),
            TableEntry::new_without_symbol(Tag::Derived, "Data {{ name: {=?:?} }}".to_owned()),
            TableEntry::new_without_symbol(Tag::Info, "{=[?]:a}".to_owned()),
        ];

        let table = test_table_with_timestamp(entries, "{=u8:us}");

        let bytes = [
            3, 0, // frame index
            2, // timestamp value of type `u8`
            1, 0, 0, 0, // number of elements in `FormatSlice`
            2, 0, // index to `Data` struct
            1, 0, // Format index to table entry: `{=[?]}`
            2, 0, 0, 0, // inner FormatSlice, number of elements in `name` field
            0, 0,   // Format index to table entry: `{=u8}`
            72,  // "H"
            105, // "i"
        ];
        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(
            frame.display(false).to_string(),
            "0.000002 INFO [Data { name: b\"Hi\" }]",
        );
    }

    #[test]
    fn bools_simple() {
        let bytes = [
            0, 0,          // index
            2,          // timestamp
            true as u8, // the logged bool value
        ];

        decode_and_expect("my bool={=bool}", &bytes, "0.000002 INFO my bool=true");
    }

    #[test]
    fn bitfields() {
        let bytes = [
            0,
            0,           // index
            2,           // timestamp
            0b1110_0101, // u8
        ];
        decode_and_expect(
            "x: {0=0..4:b}, y: {0=3..8:#b}",
            &bytes,
            "0.000002 INFO x: 101, y: 0b11100",
        );
    }

    #[test]
    fn bitfields_reverse_order() {
        let bytes = [
            0,
            0,           // index
            2,           // timestamp
            0b1101_0010, // u8
        ];
        decode_and_expect(
            "x: {0=0..7:b}, y: {0=3..5:b}",
            &bytes,
            "0.000002 INFO x: 1010010, y: 10",
        );
    }

    #[test]
    fn bitfields_different_indices() {
        let bytes = [
            0,
            0,           // index
            2,           // timestamp
            0b1111_0000, // u8
            0b1110_0101, // u8
        ];
        decode_and_expect(
            "#0: {0=0..5:b}, #1: {1=3..8:b}",
            &bytes,
            "0.000002 INFO #0: 10000, #1: 11100",
        );
    }

    #[test]
    fn bitfields_u16() {
        let bytes = [
            0,
            0, // index
            2, // timestamp
            0b1111_0000,
            0b1110_0101, // u16
        ];
        decode_and_expect("x: {0=7..12:b}", &bytes, "0.000002 INFO x: 1011");
    }

    #[test]
    fn bitfields_mixed_types() {
        let bytes = [
            0,
            0, // index
            2, // timestamp
            0b1111_0000,
            0b1110_0101, // u16
            0b1111_0001, // u8
        ];
        decode_and_expect(
            "#0: {0=7..12:b}, #1: {1=0..5:b}",
            &bytes,
            "0.000002 INFO #0: 1011, #1: 10001",
        );
    }

    #[test]
    fn bitfields_mixed() {
        let bytes = [
            0,
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
            "0.000002 INFO #0: 1011, #1: 42, #2: 10001",
        );
    }

    #[test]
    fn bitfields_across_boundaries() {
        let bytes = [
            0,
            0, // index
            2, // timestamp
            0b1101_0010,
            0b0110_0011, // u16
        ];
        decode_and_expect(
            "bitfields {0=0..7:b} {0=9..14:b}",
            &bytes,
            "0.000002 INFO bitfields 1010010 10001",
        );
    }

    #[test]
    fn bitfields_across_boundaries_diff_indices() {
        let bytes = [
            0,
            0, // index
            2, // timestamp
            0b1101_0010,
            0b0110_0011, // u16
            0b1111_1111, // truncated u16
        ];
        decode_and_expect(
            "bitfields {0=0..7:b} {0=9..14:b} {1=8..10:b}",
            &bytes,
            "0.000002 INFO bitfields 1010010 10001 11",
        );
    }

    #[test]
    fn bitfields_truncated_front() {
        let bytes = [
            0,
            0,           // index
            2,           // timestamp
            0b0110_0011, // truncated(!) u16
        ];
        decode_and_expect(
            "bitfields {0=9..14:b}",
            &bytes,
            "0.000002 INFO bitfields 10001",
        );
    }

    #[test]
    fn bitfields_non_truncated_u32() {
        let bytes = [
            0,
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
            "0.000002 INFO bitfields 11 100",
        );
    }

    #[test]
    fn bitfields_u128() {
        let bytes = [
            0,
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
        decode_and_expect("x: {0=119..124:b}", &bytes, "0.000002 INFO x: 1011");
    }

    #[test]
    fn slice() {
        let bytes = [
            0, 0, // index
            2, // timestamp
            2, 0, 0, 0, // length of the slice
            23, 42, // slice content
        ];
        decode_and_expect("x={=[u8]}", &bytes, "0.000002 INFO x=[23, 42]");
    }

    #[test]
    fn slice_with_trailing_args() {
        let bytes = [
            0, 0, // index
            2, // timestamp
            2, 0, 0, 0, // length of the slice
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
            0, 0, // index
            2, // timestamp
            5, 0, 0, 0, // length of the string
            b'W', b'o', b'r', b'l', b'd',
        ];

        decode_and_expect("Hello {=str}", &bytes, "0.000002 INFO Hello World");
    }

    #[test]
    fn string_with_trailing_data() {
        let bytes = [
            0, 0, // index
            2, // timestamp
            5, 0, 0, 0, // length of the string
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
            0, 0, // index
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
                "{=u8:us}".to_owned(),
            )),
            bitflags: Default::default(),
            encoding: Encoding::Raw,
        };

        let bytes = [
            4, 0, // string index (INFO)
            0, // timestamp
            3, 0, // string index (enum)
            1, // Some discriminant
            2, 0,  // string index (u8)
            42, // Some.0
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000000 INFO x=Some(42)");

        let bytes = [
            4, 0, // string index (INFO)
            1, // timestamp
            3, 0, // string index (enum)
            0, // None discriminant
        ];

        let frame = table.decode(&bytes).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000001 INFO x=None");
    }
}
