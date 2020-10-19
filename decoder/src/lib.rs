// NOTE: always runs on the host

use core::fmt::{self, Write as _};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::{
    error::Error,
    io, mem,
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
};

use byteorder::{ReadBytesExt, LE};
use colored::Colorize;

pub use defmt_common::Level;
use defmt_parser::{Fragment, Type};

include!(concat!(env!("OUT_DIR"), "/version.rs"));

#[derive(PartialEq, Eq, Debug)]
pub enum Tag {
    /// A format string for use with `{:?}`. Used for both primitive format strings (which are
    /// special cased to have lower indices), and user format strings created by
    /// `#[derive(Format)]`.
    Fmt,
    /// An interned string, for use with `{:istr}`.
    Str,

    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Tag {
    fn to_level(&self) -> Option<Level> {
        match self {
            Tag::Fmt | Tag::Str => None,
            Tag::Trace => Some(Level::Trace),
            Tag::Debug => Some(Level::Debug),
            Tag::Info => Some(Level::Info),
            Tag::Warn => Some(Level::Warn),
            Tag::Error => Some(Level::Error),
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

/// Interner table that holds log levels and maps format strings to indices
#[derive(Debug)]
pub struct Table {
    entries: BTreeMap<usize, TableEntry>,
}

impl Table {
    pub fn new(entries: BTreeMap<usize, TableEntry>, version: &str) -> Result<Self, String> {
        if version != DEFMT_VERSION {
            return Err(format!(
                "defmt version mismatch (firmware is using {}, host is using {}); \
                 are you using the same git version of defmt and related tools?",
                version, DEFMT_VERSION,
            ));
        }

        Ok(Self { entries })
    }

    fn _get(&self, index: usize) -> Result<(Option<Level>, &str), ()> {
        let entry = self.entries.get(&index).ok_or_else(|| ())?;
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

    pub fn indices<'s>(&'s self) -> impl Iterator<Item = usize> + 's {
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
    pub fn symbols<'s>(&'s self) -> impl Iterator<Item = &'s str> + 's {
        self.entries.values().map(|s| &*s.raw_symbol)
    }
}

/// A log frame
#[derive(Debug, PartialEq)]
pub struct Frame<'t> {
    level: Level,
    index: u64,
    // Format string
    format: &'t str,
    timestamp: u64,
    args: Vec<Arg<'t>>,
}

impl<'t> Frame<'t> {
    /// Returns a struct that will format this log frame (including message, timestamp, level,
    /// etc.).
    pub fn display(&'t self, colored: bool) -> DisplayFrame<'t> {
        DisplayFrame {
            frame: self,
            colored,
        }
    }

    /// Returns a struct that will format the message contained in this log frame.
    pub fn display_message(&'t self) -> DisplayMessage<'t> {
        DisplayMessage { frame: self }
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

pub struct DisplayMessage<'t> {
    frame: &'t Frame<'t>,
}

impl fmt::Display for DisplayMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args = format_args(&self.frame.format, &self.frame.args);
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

#[derive(Debug)]
struct Bool(AtomicBool);

impl Bool {
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
    FormatSlice {
        elements: Vec<FormatSliceElement<'t>>,
    },
    /// Slice or Array of bytes.
    Slice(Vec<u8>),
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

fn read_leb128(bytes: &mut &[u8]) -> Result<u64, DecodeError> {
    match leb128::read::unsigned(bytes) {
        Ok(val) => Ok(val),
        Err(leb128::read::Error::Overflow) => Err(DecodeError::Malformed),
        Err(leb128::read::Error::IoError(io)) => Err(io.into()),
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
) -> Result<(Frame<'t>, /*consumed: */ usize), DecodeError> {
    let len = bytes.len();
    let index = read_leb128(&mut bytes)?;
    let timestamp = read_leb128(&mut bytes)?;

    let (level, format) = table
        .get_with_level(index as usize)
        .map_err(|_| DecodeError::Malformed)?;

    let mut decoder = Decoder {
        table,
        bytes,
        format_list: None,
        bools_tbd: Vec::new(),
        below_enum: false,
    };
    let args = decoder.decode_format(format)?;

    let frame = Frame {
        level,
        index,
        format,
        timestamp,
        args,
    };

    let consumed = len - decoder.bytes.len();
    Ok((frame, consumed))
}

struct Decoder<'t, 'b> {
    table: &'t Table,
    bytes: &'b [u8],
    format_list: Option<FormatList<'t>>,
    // below an enum tags must be included
    below_enum: bool,
    bools_tbd: Vec<Arc<Bool>>,
}

const MAX_NUM_BOOL_FLAGS: usize = 8;

impl<'t, 'b> Decoder<'t, 'b> {
    /// Reads a byte of packed bools and unpacks them into `args` at the given indices.
    fn read_and_unpack_bools(&mut self) -> Result<(), DecodeError> {
        let bool_flags = self.bytes.read_u8()?;
        let mut flag_index = self.bools_tbd.len();

        for bool in self.bools_tbd.iter() {
            flag_index -= 1;

            // read out the leftmost unread bit and turn it into a boolean
            let flag_mask = 1 << flag_index;
            let nth_flag = (bool_flags & flag_mask) != 0;

            bool.set(nth_flag);
        }

        self.bools_tbd.clear();

        Ok(())
    }

    /// Gets a format string from
    /// - the `FormatList`, if it's in `Use` mode, or
    /// - from `bytes` and `table` if the `FormatList` is in `Build` mode or was not provided
    fn get_format(&mut self) -> Result<&'t str, DecodeError> {
        if let Some(FormatList::Use { formats, cursor }) = self.format_list.as_mut() {
            if let Some(format) = formats.get(*cursor) {
                *cursor += 1;
                return Ok(format);
            }
        }

        let index = read_leb128(&mut self.bytes)?;
        let format = self
            .table
            .get_without_level(index as usize)
            .map_err(|_| DecodeError::Malformed)?;

        if let Some(FormatList::Build { formats }) = self.format_list.as_mut() {
            if !self.below_enum {
                formats.push(format)
            }
        }
        Ok(format)
    }

    fn get_variant(&mut self, format: &'t str) -> Result<&'t str, DecodeError> {
        assert!(format.contains("|"));
        let discriminant = self.bytes.read_u8()?;

        // NOTE nesting of enums, like "A|B(C|D)" is not possible; indirection is
        // required: "A|B({:?})" where "{:?}" -> "C|D"
        format
            .split('|')
            .nth(usize::from(discriminant))
            .ok_or(DecodeError::Malformed)
    }

    /// Decodes arguments from the stream, according to `format`.
    fn decode_format(&mut self, format: &str) -> Result<Vec<Arg<'t>>, DecodeError> {
        let mut args = vec![]; // will contain the deserialized arguments on return
        let mut params = defmt_parser::parse(format)
            .map_err(|_| DecodeError::Malformed)?
            .iter()
            .filter_map(|frag| match frag {
                Fragment::Parameter(param) => Some(param.clone()),
                Fragment::Literal(_) => None,
            })
            .collect::<Vec<_>>();

        // sort & dedup to ensure that format string args can be addressed by index too
        params.sort_by(|a, b| {
            if a.index == b.index {
                match (&a.ty, &b.ty) {
                    (Type::BitField(a_range), Type::BitField(b_range)) => {
                        b_range.end.cmp(&a_range.end)
                    }
                    _ => Ordering::Equal,
                }
            } else {
                a.index.cmp(&b.index)
            }
        });

        params.dedup_by(|a, b| {
            if a.index == b.index {
                match (&a.ty, &b.ty) {
                    (Type::BitField(a_range), Type::BitField(b_range)) => a_range.end < b_range.end,
                    /* reusing an arg for bitfield- and non bitfield params is not allowed */
                    (Type::BitField(_), _) => unreachable!(),
                    (_, Type::BitField(_)) => unreachable!(),
                    _ => true,
                }
            } else {
                false
            }
        });

        for param in &params {
            match &param.ty {
                Type::U8 => {
                    let data = self.bytes.read_u8()?;
                    args.push(Arg::Uxx(data as u64));
                }

                Type::Bool => {
                    let arc = Arc::new(Bool::FALSE);
                    args.push(Arg::Bool(arc.clone()));
                    self.bools_tbd.push(arc.clone());
                    if self.bools_tbd.len() == MAX_NUM_BOOL_FLAGS {
                        // reached end of compression block: sprinkle values into args
                        self.read_and_unpack_bools()?;
                    }
                }

                Type::FormatSlice => {
                    let num_elements = read_leb128(&mut self.bytes)? as usize;

                    let arg = if num_elements == 0 {
                        Arg::FormatSlice { elements: vec![] }
                    } else {
                        let format = self.get_format()?;

                        // let variant_format = if
                        let is_enum = format.contains('|');
                        let below_enum = self.below_enum;

                        if is_enum {
                            self.below_enum = true;
                        }

                        let mut elements = Vec::with_capacity(num_elements);
                        let mut formats = vec![];
                        let mut cursor = 0;
                        for i in 0..num_elements {
                            let is_first = i == 0;

                            let format = if is_enum {
                                self.get_variant(format)?
                            } else {
                                format
                            };

                            let args = if let Some(list) = &mut self.format_list {
                                match list {
                                    FormatList::Use { .. } => self.decode_format(format)?,

                                    FormatList::Build { formats } => {
                                        if is_first {
                                            cursor = formats.len();
                                            self.decode_format(format)?
                                        } else {
                                            let formats = formats.clone();
                                            let old = mem::replace(
                                                &mut self.format_list,
                                                Some(FormatList::Use { formats, cursor }),
                                            );
                                            let args = self.decode_format(format)?;
                                            self.format_list = old;
                                            args
                                        }
                                    }
                                }
                            } else {
                                if is_first {
                                    let mut old = mem::replace(
                                        &mut self.format_list,
                                        Some(FormatList::Build { formats }),
                                    );
                                    let args = self.decode_format(format)?;
                                    mem::swap(&mut self.format_list, &mut old);
                                    formats = match old {
                                        Some(FormatList::Build { formats, .. }) => formats,
                                        _ => unreachable!(),
                                    };
                                    args
                                } else {
                                    let formats = formats.clone();
                                    let old = mem::replace(
                                        &mut self.format_list,
                                        Some(FormatList::Use { formats, cursor: 0 }),
                                    );
                                    let args = self.decode_format(format)?;
                                    self.format_list = old;
                                    args
                                }
                            };

                            elements.push(FormatSliceElement { format, args });
                        }

                        if is_enum {
                            self.below_enum = below_enum;
                        }

                        Arg::FormatSlice { elements }
                    };

                    args.push(arg);
                }
                Type::Format => {
                    let format = self.get_format()?;

                    if format.contains('|') {
                        // enum
                        let variant = self.get_variant(format)?;
                        let below_enum = self.below_enum;
                        self.below_enum = true;
                        let inner_args = self.decode_format(variant)?;
                        self.below_enum = below_enum;
                        args.push(Arg::Format {
                            format: variant,
                            args: inner_args,
                        });
                    } else {
                        let inner_args = self.decode_format(format)?;
                        args.push(Arg::Format {
                            format,
                            args: inner_args,
                        });
                    }
                }
                Type::I16 => {
                    let data = self.bytes.read_i16::<LE>()?;
                    args.push(Arg::Ixx(data as i64));
                }
                Type::I32 => {
                    let data = self.bytes.read_i32::<LE>()?;
                    args.push(Arg::Ixx(data as i64));
                }
                Type::I8 => {
                    let data = self.bytes.read_i8()?;
                    args.push(Arg::Ixx(data as i64));
                }
                Type::Isize => {
                    // Signed isize is encoded in zigzag-encoding.
                    let unsigned = read_leb128(&mut self.bytes)?;
                    args.push(Arg::Ixx(zigzag_decode(unsigned)))
                }
                Type::U16 => {
                    let data = self.bytes.read_u16::<LE>()?;
                    args.push(Arg::Uxx(data as u64));
                }
                Type::U24 => {
                    let data_low = self.bytes.read_u8()?;
                    let data_high = self.bytes.read_u16::<LE>()?;
                    let data = data_low as u64 | (data_high as u64) << 8;
                    args.push(Arg::Uxx(data as u64));
                }
                Type::U32 => {
                    let data = self.bytes.read_u32::<LE>()?;
                    args.push(Arg::Uxx(data as u64));
                }
                Type::Usize => {
                    let unsigned = read_leb128(&mut self.bytes)?;
                    args.push(Arg::Uxx(unsigned))
                }
                Type::F32 => {
                    let data = self.bytes.read_u32::<LE>()?;
                    args.push(Arg::F32(f32::from_bits(data)));
                }
                Type::BitField(range) => {
                    let data: u64;

                    match range.end {
                        0..=8 => {
                            data = self.bytes.read_u8()? as u64;
                        }
                        0..=16 => {
                            data = self.bytes.read_u16::<LE>()? as u64;
                        }
                        0..=24 => {
                            data = self.bytes.read_u24::<LE>()? as u64;
                        }
                        0..=32 => {
                            data = self.bytes.read_u32::<LE>()? as u64;
                        }
                        _ => {
                            unreachable!();
                        }
                    }

                    args.push(Arg::Uxx(data));
                }
                Type::Str => {
                    let str_len = read_leb128(&mut self.bytes)? as usize;
                    let mut arg_str_bytes = vec![];

                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..str_len {
                        arg_str_bytes.push(self.bytes.read_u8()?);
                    }

                    // convert to utf8 (no copy)
                    let arg_str = String::from_utf8(arg_str_bytes).unwrap();

                    args.push(Arg::Str(arg_str));
                }
                Type::IStr => {
                    let str_index = read_leb128(&mut self.bytes)? as usize;

                    let string = self
                        .table
                        .get_without_level(str_index as usize)
                        .map_err(|_| DecodeError::Malformed)?;

                    args.push(Arg::IStr(string));
                }
                Type::Slice => {
                    // only supports byte slices
                    let num_elements = read_leb128(&mut self.bytes)? as usize;
                    let mut arg_slice = vec![];

                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..num_elements {
                        arg_slice.push(self.bytes.read_u8()?);
                    }
                    args.push(Arg::Slice(arg_slice.to_vec()));
                }
                Type::Array(len) => {
                    let mut arg_slice = vec![];
                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..*len {
                        arg_slice.push(self.bytes.read_u8()?);
                    }
                    args.push(Arg::Slice(arg_slice.to_vec()));
                }
            }
        }

        if self.bools_tbd.len() > 0 {
            // flush end of compression block
            self.read_and_unpack_bools()?;
        }

        Ok(args)
    }
}

/// List of format strings; used when decoding a `FormatSlice` (`{:[?]}`) argument
#[derive(Debug)]
enum FormatList<'t> {
    /// Build the list; used when decoding the first element
    Build { formats: Vec<&'t str> },
    /// Use the list; used when decoding the rest of elements
    Use {
        formats: Vec<&'t str>,
        cursor: usize,
    },
}

fn format_args(format: &str, args: &[Arg]) -> String {
    format_args_real(format, args).unwrap() // cannot fail, we only write to a `String`
}

fn format_args_real(format: &str, args: &[Arg]) -> Result<String, fmt::Error> {
    let params = defmt_parser::parse(format).unwrap();
    let mut buf = String::new();
    for param in params {
        match param {
            Fragment::Literal(lit) => {
                buf.push_str(&lit);
            }
            Fragment::Parameter(param) => {
                match &args[param.index] {
                    Arg::Bool(x) => write!(buf, "{}", x)?,
                    Arg::F32(x) => write!(buf, "{}", ryu::Buffer::new().format(*x))?,
                    Arg::Uxx(x) => {
                        match param.ty {
                            Type::BitField(range) => {
                                let left_zeroes = mem::size_of::<u64>() * 8 - range.end as usize;
                                let right_zeroes = left_zeroes + range.start as usize;
                                // isolate the desired bitfields
                                let bitfields = (*x << left_zeroes) >> right_zeroes;
                                write!(&mut buf, "{:#b}", bitfields)?
                            }
                            _ => write!(buf, "{}", x)?,
                        }
                    }
                    Arg::Ixx(x) => write!(buf, "{}", x)?,
                    Arg::Str(x) => write!(buf, "{}", x)?,
                    Arg::IStr(x) => write!(buf, "{}", x)?,
                    Arg::Format { format, args } => buf.push_str(&format_args(format, args)),
                    Arg::FormatSlice { elements } => {
                        buf.write_str("[")?;
                        let mut is_first = true;
                        for element in elements {
                            if !is_first {
                                buf.write_str(", ")?;
                            }
                            is_first = false;
                            buf.write_str(&format_args(element.format, &element.args))?;
                        }
                        buf.write_str("]")?;
                    }
                    Arg::Slice(x) => write!(buf, "{:?}", x)?,
                }
            }
        }
    }
    Ok(buf)
}

fn zigzag_decode(unsigned: u64) -> i64 {
    (unsigned >> 1) as i64 ^ -((unsigned & 1) as i64)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use defmt_common::Level;

    use super::*;
    use crate::Arg;

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

        let table = Table { entries };

        let frame = super::decode(&bytes, &table).unwrap().0;
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
            TableEntry::new_without_symbol(Tag::Debug, "The answer is {:u8}!".to_owned()),
        );
        // [IDX, TS, 42]
        //           ^^
        //entries.insert(2, "The answer is {0:u8} {1:u16}!".to_owned());

        let table = Table { entries };

        let bytes = [0, 1];
        //     index ^  ^ timestamp

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    index: 0,
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
                    index: 1,
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
        entries.insert(0, TableEntry::new_without_symbol(Tag::Info, FMT.to_owned()));

        let table = Table { entries };

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
                    index: 0,
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
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "The answer is {0:u8} {0:u8}!".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(
                Tag::Info,
                "The answer is {1:u16} {0:u8} {1:u16}!".to_owned(),
            ),
        );

        let table = Table { entries };
        let bytes = [
            0,  // index
            2,  // timestamp
            42, // argument
        ];

        assert_eq!(
            super::decode(&bytes, &table),
            Ok((
                Frame {
                    index: 0,
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
                    index: 1,
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
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "x={:?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(Tag::Fmt, "Foo {{ x: {:u8} }}".to_owned()),
        );

        let table = Table { entries };

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
                    index: 0,
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
        entries.insert(
            0,
            TableEntry::new_without_symbol(Tag::Info, "x={:?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(Tag::Fmt, "Foo {{ x: {:u8} }}".to_owned()),
        );

        let table = Table { entries };

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
    fn bools_simple() {
        let bytes = [
            0,          // index
            2,          // timestamp
            true as u8, // the logged bool value
        ];

        decode_and_expect("my bool={:bool}", &bytes, "0.000002 INFO my bool=true");
    }

    #[test]
    fn bools_max_capacity() {
        let bytes = [
            0,           // index
            2,           // timestamp
            0b0110_0001, // the first 8 logged bool values
        ];

        decode_and_expect(
            "bool capacity {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
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
            "bool overflow {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
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
            "bool overflow {:bool} {:u8} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool}",
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
            "bool overflow {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:bool} {:u8} {:bool}",
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
            "hidden bools {:bool} {:u8} {:bool} {:bool}",
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
            "no trailing bools {:bool} {:u8}",
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
            TableEntry::new_without_symbol(Tag::Info, "{:bool} {:?}".to_owned()),
        );
        entries.insert(
            1,
            TableEntry::new_without_symbol(
                Tag::Fmt,
                "Flags {{ a: {:bool}, b: {:bool}, c: {:bool} }}".to_owned(),
            ),
        );

        let table = Table { entries };

        let bytes = [
            0,      // index
            2,      // timestamp
            1,      // index of Flags { a: {:bool}, b: {:bool}, c: {:bool} }
            0b1101, // 4 packed bools
        ];

        let frame = super::decode(&bytes, &table).unwrap().0;
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
            "x: {0:0..4}, y: {0:3..8}",
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
            "x: {0:0..7}, y: {0:3..5}",
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
            "#0: {0:0..5}, #1: {1:3..8}",
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
        decode_and_expect("x: {0:7..12}", &bytes, "0.000002 INFO x: 0b1011");
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
            "#0: {0:7..12}, #1: {1:0..5}",
            &bytes,
            "0.000002 INFO #0: 0b1011, #1: 0b10001",
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
            "bitfields {0:0..7} {0:9..14}",
            &bytes,
            "0.000002 INFO bitfields 0b1010010 0b10001",
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

    #[test]
    fn option() {
        let mut entries = BTreeMap::new();
        entries.insert(
            4,
            TableEntry::new_without_symbol(Tag::Info, "x={:?}".to_owned()),
        );
        entries.insert(
            3,
            TableEntry::new_without_symbol(Tag::Fmt, "None|Some({:?})".to_owned()),
        );
        entries.insert(
            2,
            TableEntry::new_without_symbol(Tag::Fmt, "{:u8}".to_owned()),
        );

        let table = Table { entries };

        let bytes = [
            4,  // string index (INFO)
            0,  // timestamp
            3,  // string index (enum)
            1,  // Some discriminant
            2,  // string index (u8)
            42, // Some.0
        ];

        let frame = super::decode(&bytes, &table).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000000 INFO x=Some(42)");

        let bytes = [
            4, // string index (INFO)
            1, // timestamp
            3, // string index (enum)
            0, // None discriminant
        ];

        let frame = super::decode(&bytes, &table).unwrap().0;
        assert_eq!(frame.display(false).to_string(), "0.000001 INFO x=None");
    }
}
