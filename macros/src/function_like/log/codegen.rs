use defmt_parser::{Fragment, Type};
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use proc_macro_error::abort;
use quote::{format_ident, quote};

pub(crate) struct Codegen {
    pub(crate) pats: Vec<Ident2>,
    pub(crate) exprs: Vec<TokenStream2>,
}

impl Codegen {
    pub(crate) fn new(fragments: &[Fragment<'_>], num_args: usize, span: Span2) -> Self {
        let parsed_params = fragments
            .iter()
            .filter_map(|frag| match frag {
                Fragment::Parameter(param) => Some(param.clone()),
                Fragment::Literal(_) => None,
            })
            .collect::<Vec<_>>();

        let actual_argument_count = parsed_params
            .iter()
            .map(|param| param.index + 1)
            .max()
            .unwrap_or(0);

        let mut exprs = vec![];
        let mut pats = vec![];

        for i in 0..actual_argument_count {
            let arg = format_ident!("arg{}", i);
            // find first use of this argument and return its type
            let param = parsed_params.iter().find(|param| param.index == i).unwrap();
            match param.ty {
                Type::I8 => exprs.push(quote!(defmt::export::i8(#arg))),
                Type::I16 => exprs.push(quote!(defmt::export::i16(#arg))),
                Type::I32 => exprs.push(quote!(defmt::export::i32(#arg))),
                Type::I64 => exprs.push(quote!(defmt::export::i64(#arg))),
                Type::I128 => exprs.push(quote!(defmt::export::i128(#arg))),
                Type::Isize => exprs.push(quote!(defmt::export::isize(#arg))),

                Type::U8 => exprs.push(quote!(defmt::export::u8(#arg))),
                Type::U16 => exprs.push(quote!(defmt::export::u16(#arg))),
                Type::U32 => exprs.push(quote!(defmt::export::u32(#arg))),
                Type::U64 => exprs.push(quote!(defmt::export::u64(#arg))),
                Type::U128 => exprs.push(quote!(defmt::export::u128(#arg))),
                Type::Usize => exprs.push(quote!(defmt::export::usize(#arg))),

                Type::F32 => exprs.push(quote!(defmt::export::f32(#arg))),
                Type::F64 => exprs.push(quote!(defmt::export::f64(#arg))),

                Type::Bool => exprs.push(quote!(defmt::export::bool(#arg))),

                Type::Str => exprs.push(quote!(defmt::export::str(#arg))),
                Type::IStr => exprs.push(quote!(defmt::export::istr(#arg))),
                Type::Char => exprs.push(quote!(defmt::export::char(#arg))),

                Type::Format => exprs.push(quote!(defmt::export::fmt(#arg))),
                Type::FormatSlice => exprs.push(quote!(defmt::export::fmt_slice(#arg))),
                Type::FormatArray(len) => exprs.push(quote!(defmt::export::fmt_array({
                    let tmp: &[_; #len] = #arg;
                    tmp
                }))),

                Type::Debug => exprs.push(quote!(defmt::export::debug(#arg))),
                Type::Display => exprs.push(quote!(defmt::export::display(#arg))),
                Type::FormatSequence => unreachable!(),

                Type::U8Slice => exprs.push(quote!(defmt::export::slice(#arg))),
                // We cast to the expected array type (which should be a no-op cast) to provoke
                // a type mismatch error on mismatched lengths:
                // ```
                // error[E0308]: mismatched types
                //   --> src/bin/log.rs:20:5
                //    |
                // 20 |     defmt::info!("🍕 array {:[u8; 3]}", [3, 14]);
                //    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                //    |     |
                //    |     expected an array with a fixed size of 3 elements, found one with 2 elements
                //    |     expected due to this
                // ```
                Type::U8Array(len) => exprs.push(quote!(defmt::export::u8_array({
                    let tmp: &[u8; #len] = #arg;
                    tmp
                }))),
                Type::BitField(_) => {
                    let all_bitfields = parsed_params.iter().filter(|param| param.index == i);
                    let (smallest_bit_index, largest_bit_index) =
                        defmt_parser::get_max_bitfield_range(all_bitfields).unwrap();

                    // indices of the lowest and the highest octet which contains bitfield-relevant data
                    let lowest_byte = smallest_bit_index / 8;
                    let highest_byte = (largest_bit_index - 1) / 8;
                    let truncated_sz = highest_byte - lowest_byte + 1; // in bytes

                    // shift away unneeded lower octet
                    // TODO: create helper for shifting because readability
                    match truncated_sz {
                        1 => exprs.push(quote!(defmt::export::u8(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))),
                        2 => exprs.push(quote!(defmt::export::u16(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))),
                        3..=4 => exprs.push(quote!(defmt::export::u32(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))),
                        5..=8 => exprs.push(quote!(defmt::export::u64(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))),
                        9..=16 => exprs.push(quote!(defmt::export::u128(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))),
                        _ => unreachable!(),
                    }
                }
            }
            pats.push(arg);
        }

        if num_args != actual_argument_count {
            let mut only = "";
            if num_args < actual_argument_count {
                only = "only ";
            }

            abort!(
                span,
                "format string requires {} arguments but {}{} were provided",
                actual_argument_count,
                only,
                num_args
            )
        }

        Codegen { pats, exprs }
    }
}
