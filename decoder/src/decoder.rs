use std::{
    convert::{TryFrom, TryInto},
    mem,
    ops::Range,
};

use crate::{Arg, DecodeError, FormatSliceElement, Table};
use byteorder::{ReadBytesExt, LE};
use defmt_parser::{get_max_bitfield_range, Fragment, Parameter, Type};

pub(crate) struct Decoder<'t, 'b> {
    table: &'t Table,
    pub bytes: &'b [u8],
}

impl<'t, 'b> Decoder<'t, 'b> {
    pub fn new(table: &'t Table, bytes: &'b [u8]) -> Self {
        Self { table, bytes }
    }

    /// Sort and deduplicate `params` so that they can be interpreted correctly during decoding
    fn prepare_params(&self, params: &mut Vec<Parameter>) {
        // deduplicate bitfields by merging them by index
        merge_bitfields(params);

        // sort & dedup to ensure that format string args can be addressed by index too
        params.sort_by(|a, b| a.index.cmp(&b.index));
        params.dedup_by(|a, b| a.index == b.index);
    }

    /// Gets a format string from `bytes` and `table`
    fn get_format(&mut self) -> Result<&'t str, DecodeError> {
        let index = self.bytes.read_u16::<LE>()? as usize;
        let format = self
            .table
            .get_without_level(index as usize)
            .map_err(|_| DecodeError::Malformed)?;

        Ok(format)
    }

    fn get_variant(&mut self, format: &'t str) -> Result<&'t str, DecodeError> {
        assert!(format.contains('|'));
        // NOTE nesting of enums, like "A|B(C|D)" is not possible; indirection is
        // required: "A|B({:?})" where "{:?}" -> "C|D"
        let num_variants = format.chars().filter(|c| *c == '|').count();

        let discriminant: usize = if u8::try_from(num_variants).is_ok() {
            self.bytes.read_u8()?.into()
        } else if u16::try_from(num_variants).is_ok() {
            self.bytes.read_u16::<LE>()?.into()
        } else if u32::try_from(num_variants).is_ok() {
            self.bytes
                .read_u32::<LE>()?
                .try_into()
                .map_err(|_| DecodeError::Malformed)?
        } else if u64::try_from(num_variants).is_ok() {
            self.bytes
                .read_u64::<LE>()?
                .try_into()
                .map_err(|_| DecodeError::Malformed)?
        } else {
            return Err(DecodeError::Malformed);
        };

        format
            .split('|')
            .nth(discriminant)
            .ok_or(DecodeError::Malformed)
    }

    fn decode_format_slice(
        &mut self,
        num_elements: usize,
    ) -> Result<Vec<FormatSliceElement<'t>>, DecodeError> {
        let format = self.get_format()?;
        let is_enum = format.contains('|');

        let mut elements = Vec::with_capacity(num_elements);
        for i in 0..num_elements {
            let format = if is_enum {
                self.get_variant(format)?
            } else {
                format
            };
            let args = self.decode_format(format)?;
            elements.push(FormatSliceElement { format, args });
        }

        Ok(elements)
    }

    /// Decodes arguments from the stream, according to `format`.
    pub fn decode_format(&mut self, format: &str) -> Result<Vec<Arg<'t>>, DecodeError> {
        let mut args = vec![]; // will contain the deserialized arguments on return
        let mut params = defmt_parser::parse(format, defmt_parser::ParserMode::ForwardsCompatible)
            .map_err(|_| DecodeError::Malformed)?
            .iter()
            .filter_map(|frag| match frag {
                Fragment::Parameter(param) => Some(param.clone()),
                Fragment::Literal(_) => None,
            })
            .collect::<Vec<_>>();

        self.prepare_params(&mut params);

        for param in &params {
            match &param.ty {
                Type::I8 => args.push(Arg::Ixx(self.bytes.read_i8()? as i128)),
                Type::I16 => args.push(Arg::Ixx(self.bytes.read_i16::<LE>()? as i128)),
                Type::I32 => args.push(Arg::Ixx(self.bytes.read_i32::<LE>()? as i128)),
                Type::I64 => args.push(Arg::Ixx(self.bytes.read_i64::<LE>()? as i128)),
                Type::I128 => args.push(Arg::Ixx(self.bytes.read_i128::<LE>()?)),
                Type::Isize => args.push(Arg::Ixx(self.bytes.read_i32::<LE>()? as i128)),
                Type::U8 => args.push(Arg::Uxx(self.bytes.read_u8()? as u128)),
                Type::U16 => args.push(Arg::Uxx(self.bytes.read_u16::<LE>()? as u128)),
                Type::U32 => args.push(Arg::Uxx(self.bytes.read_u32::<LE>()? as u128)),
                Type::U64 => args.push(Arg::Uxx(self.bytes.read_u64::<LE>()? as u128)),
                Type::U128 => args.push(Arg::Uxx(self.bytes.read_u128::<LE>()? as u128)),
                Type::Usize => args.push(Arg::Uxx(self.bytes.read_u32::<LE>()? as u128)),
                Type::F32 => args.push(Arg::F32(f32::from_bits(self.bytes.read_u32::<LE>()?))),
                Type::F64 => args.push(Arg::F64(f64::from_bits(self.bytes.read_u64::<LE>()?))),
                Type::Bool => args.push(Arg::Bool(match self.bytes.read_u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(DecodeError::Malformed),
                })),
                Type::FormatSlice => {
                    let num_elements = self.bytes.read_u32::<LE>()? as usize;
                    let elements = self.decode_format_slice(num_elements)?;
                    args.push(Arg::FormatSlice { elements });
                }
                Type::Format => {
                    let format = self.get_format()?;

                    if format.contains('|') {
                        // enum
                        let variant = self.get_variant(format)?;
                        let inner_args = self.decode_format(variant)?;
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
                Type::BitField(range) => {
                    let mut data: u128;
                    let lowest_byte = range.start / 8;
                    let highest_byte = (range.end - 1) / 8; // -1, because `range` is range-exclusive
                    let size_after_truncation = highest_byte - lowest_byte + 1; // in octets

                    data = match size_after_truncation {
                        1 => self.bytes.read_u8()? as u128,
                        2 => self.bytes.read_u16::<LE>()? as u128,
                        3..=4 => self.bytes.read_u32::<LE>()? as u128,
                        5..=8 => self.bytes.read_u64::<LE>()? as u128,
                        9..=16 => self.bytes.read_u128::<LE>()? as u128,
                        _ => unreachable!(),
                    };

                    data <<= lowest_byte * 8;

                    args.push(Arg::Uxx(data));
                }
                Type::Str => {
                    let str_len = self.bytes.read_u32::<LE>()? as usize;
                    let mut arg_str_bytes = vec![];

                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..str_len {
                        arg_str_bytes.push(self.bytes.read_u8()?);
                    }

                    // convert to utf8 (no copy)
                    let arg_str =
                        String::from_utf8(arg_str_bytes).map_err(|_| DecodeError::Malformed)?;

                    args.push(Arg::Str(arg_str));
                }
                Type::IStr => {
                    let str_index = self.bytes.read_u16::<LE>()? as usize;

                    let string = self
                        .table
                        .get_without_level(str_index as usize)
                        .map_err(|_| DecodeError::Malformed)?;

                    args.push(Arg::IStr(string));
                }
                Type::U8Slice => {
                    // only supports byte slices
                    let num_elements = self.bytes.read_u32::<LE>()? as usize;
                    let mut arg_slice = vec![];

                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..num_elements {
                        arg_slice.push(self.bytes.read_u8()?);
                    }
                    args.push(Arg::Slice(arg_slice.to_vec()));
                }
                Type::U8Array(len) => {
                    let mut arg_slice = vec![];
                    // note: went for the suboptimal but simple solution; optimize if necessary
                    for _ in 0..*len {
                        arg_slice.push(self.bytes.read_u8()?);
                    }
                    args.push(Arg::Slice(arg_slice.to_vec()));
                }
                Type::FormatArray(len) => {
                    let elements = self.decode_format_slice(*len)?;
                    args.push(Arg::FormatSlice { elements });
                }
                Type::Char => {
                    let data = self.bytes.read_u32::<LE>()?;
                    let c = std::char::from_u32(data).ok_or(DecodeError::Malformed)?;
                    args.push(Arg::Char(c));
                }
                Type::Debug | Type::Display => {
                    // UTF-8 stream without a prefix length, terminated with `0xFF`.

                    let end = self
                        .bytes
                        .iter()
                        .position(|b| *b == 0xff)
                        .ok_or(DecodeError::UnexpectedEof)?;
                    let data = core::str::from_utf8(&self.bytes[..end])
                        .map_err(|_| DecodeError::Malformed)?;
                    self.bytes = &self.bytes[end + 1..];

                    args.push(Arg::Preformatted(data.into()));
                }
                Type::FormatSequence => {
                    loop {
                        let index = self.bytes.read_u16::<LE>()? as usize;
                        if index == 0 {
                            break;
                        }

                        let format = self
                            .table
                            .get_without_level(index as usize)
                            .map_err(|_| DecodeError::Malformed)?;

                        if format.contains('|') {
                            // enum
                            let variant = self.get_variant(format)?;
                            let inner_args = self.decode_format(variant)?;
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
                }
            }
        }

        Ok(args)
    }
}

/// Note that this will not change the Bitfield params in place, i.e. if `params` was sorted before
/// a call to this function, it won't be afterwards.
fn merge_bitfields(params: &mut Vec<Parameter>) {
    if params.is_empty() {
        return;
    }

    let mut merged_bitfields = Vec::new();

    let max_index: usize = *params.iter().map(|param| &param.index).max().unwrap();

    for index in 0..=max_index {
        let mut bitfields_with_index = params
            .iter()
            .filter(
                |param| matches!((param.index, &param.ty), (i, Type::BitField(_)) if i == index),
            )
            .peekable();

        if bitfields_with_index.peek().is_some() {
            let (smallest, largest) = get_max_bitfield_range(bitfields_with_index).unwrap();

            // create new merged bitfield for this index
            merged_bitfields.push(Parameter {
                index,
                ty: Type::BitField(Range {
                    start: smallest,
                    end: largest,
                }),
                hint: None, // don't care
            });

            // remove old bitfields with this index
            // TODO refactor when `drain_filter()` is stable
            let mut i = 0;
            while i != params.len() {
                match &params[i].ty {
                    Type::BitField(_) => {
                        if params[i].index == index {
                            params.remove(i);
                        } else {
                            i += 1; // we haven't removed a bitfield -> move i forward
                        }
                    }
                    _ => {
                        i += 1; // we haven't removed a bitfield -> move i forward
                    }
                }
            }
        }
    }

    // add merged bitfields to unsorted params
    params.append(&mut merged_bitfields);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_bitfields_simple() {
        let mut params = vec![
            Parameter {
                index: 0,
                ty: Type::BitField(0..3),
                hint: None,
            },
            Parameter {
                index: 0,
                ty: Type::BitField(4..7),
                hint: None,
            },
        ];

        merge_bitfields(&mut params);
        assert_eq!(
            params,
            vec![Parameter {
                index: 0,
                ty: Type::BitField(0..7),
                hint: None,
            }]
        );
    }

    #[test]
    fn merge_bitfields_overlap() {
        let mut params = vec![
            Parameter {
                index: 0,
                ty: Type::BitField(1..3),
                hint: None,
            },
            Parameter {
                index: 0,
                ty: Type::BitField(2..5),
                hint: None,
            },
        ];

        merge_bitfields(&mut params);
        assert_eq!(
            params,
            vec![Parameter {
                index: 0,
                ty: Type::BitField(1..5),
                hint: None,
            }]
        );
    }

    #[test]
    fn merge_bitfields_multiple_indices() {
        let mut params = vec![
            Parameter {
                index: 0,
                ty: Type::BitField(0..3),
                hint: None,
            },
            Parameter {
                index: 1,
                ty: Type::BitField(1..3),
                hint: None,
            },
            Parameter {
                index: 1,
                ty: Type::BitField(4..5),
                hint: None,
            },
        ];

        merge_bitfields(&mut params);
        assert_eq!(
            params,
            vec![
                Parameter {
                    index: 0,
                    ty: Type::BitField(0..3),
                    hint: None,
                },
                Parameter {
                    index: 1,
                    ty: Type::BitField(1..5),
                    hint: None,
                }
            ]
        );
    }

    #[test]
    fn merge_bitfields_overlap_non_consecutive_indices() {
        let mut params = vec![
            Parameter {
                index: 0,
                ty: Type::BitField(0..3),
                hint: None,
            },
            Parameter {
                index: 1,
                ty: Type::U8,
                hint: None,
            },
            Parameter {
                index: 2,
                ty: Type::BitField(1..4),
                hint: None,
            },
            Parameter {
                index: 2,
                ty: Type::BitField(4..5),
                hint: None,
            },
        ];

        merge_bitfields(&mut params);
        // note: current implementation appends merged bitfields to the end. this is not a must
        assert_eq!(
            params,
            vec![
                Parameter {
                    index: 1,
                    ty: Type::U8,
                    hint: None,
                },
                Parameter {
                    index: 0,
                    ty: Type::BitField(0..3),
                    hint: None,
                },
                Parameter {
                    index: 2,
                    ty: Type::BitField(1..5),
                    hint: None,
                }
            ]
        );
    }
}
