use defmt_parser::{Fragment, Parameter, Type};
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_quote, punctuated::Punctuated, Expr, Token};

use super::args::FormatArg;

pub(crate) struct Codegen {
    pub(crate) exprs: Vec<TokenStream2>,
    pub(crate) patterns: Vec<Ident2>,
}

/// Number of arguments referenced by the format string's parameters.
fn expected_arg_count<'a>(params: impl IntoIterator<Item = &'a Parameter>) -> usize {
    params
        .into_iter()
        .map(|param| param.index + 1)
        .max()
        .unwrap_or(0)
}

/// Resolves the given formatting arguments against the format string's parameters,
/// returning one expression per argument index, in index order.
///
/// Positional arguments are used as-is. Named parameters take the matching `name = expr`
/// argument if one was given; otherwise the identifier is captured from the surrounding
/// scope, like `core::format_args!` does for `format!("{owner}")`.
pub(crate) fn resolve_args(
    fragments: &[Fragment<'_>],
    format_span: Span2,
    args: Option<Punctuated<FormatArg, Token![,]>>,
) -> syn::Result<Vec<Expr>> {
    let params = fragments
        .iter()
        .filter_map(|frag| match frag {
            Fragment::Parameter(param) => Some(param),
            Fragment::Literal(_) => None,
        })
        .collect::<Vec<_>>();

    let arg_count = expected_arg_count(params.iter().copied());

    // The parser assigns named parameters the indices after all positional ones.
    let mut names: Vec<Option<&str>> = vec![None; arg_count];
    for param in &params {
        if let Some(name) = &param.name {
            names[param.index] = Some(name);
        }
    }
    let named_count = names.iter().filter(|name| name.is_some()).count();
    let positional_count = arg_count - named_count;

    // Split the given arguments into positional and named ones.
    let mut positional_exprs = Vec::new();
    let mut named_exprs: Vec<(syn::Ident, Expr)> = Vec::new();
    for arg in args.into_iter().flatten() {
        match arg {
            FormatArg::Positional(expr) => {
                if !named_exprs.is_empty() {
                    return Err(syn::Error::new_spanned(
                        &expr,
                        "positional arguments cannot follow named arguments",
                    ));
                }
                positional_exprs.push(expr);
            }
            FormatArg::Named(ident, expr) => {
                if named_exprs.iter().any(|(name, _)| *name == ident) {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("duplicate argument named `{ident}`"),
                    ));
                }
                named_exprs.push((ident, expr));
            }
        }
    }

    if positional_exprs.len() != positional_count {
        let given = positional_exprs.len();
        let mut only = "";
        if given < positional_count {
            only = "only ";
        }
        // Keep the pre-named-arguments wording when no named arguments are involved.
        let kind = match named_count == 0 && named_exprs.is_empty() {
            true => "arguments",
            false => "positional arguments",
        };
        return Err(syn::Error::new(
            format_span,
            format!(
                "format string requires {positional_count} {kind} but {only}{given} were provided"
            ),
        ));
    }

    // Assemble the final argument list, resolving each named parameter to the given
    // `name = expr` or to a capture of `name` from the surrounding scope.
    let mut exprs = positional_exprs;
    for name in &names[positional_count..] {
        let name = name.unwrap(/* named indices are contiguous after positionals */);
        if let Some(position) = named_exprs.iter().position(|(ident, _)| ident == name) {
            exprs.push(named_exprs.remove(position).1);
        } else {
            let mut ident = syn::parse_str::<syn::Ident>(name).map_err(|_| {
                syn::Error::new(
                    format_span,
                    format!("invalid argument name `{name}`: keywords cannot be captured"),
                )
            })?;
            ident.set_span(format_span);
            exprs.push(parse_quote!(#ident));
        }
    }

    // All remaining named arguments were not referenced by the format string.
    if let Some((ident, _)) = named_exprs.first() {
        return Err(syn::Error::new(
            ident.span(),
            format!("named argument `{ident}` is not used in this format string"),
        ));
    }

    Ok(exprs)
}

impl Codegen {
    pub(crate) fn new(
        fragments: &[Fragment<'_>],
        given_arg_count: usize,
        span: Span2,
    ) -> syn::Result<Self> {
        let params = fragments
            .iter()
            .filter_map(|frag| match frag {
                Fragment::Parameter(param) => Some(param.clone()),
                Fragment::Literal(_) => None,
            })
            .collect::<Vec<_>>();

        let expected_arg_count = expected_arg_count(&params);

        if given_arg_count != expected_arg_count {
            let mut only = "";
            if given_arg_count < expected_arg_count {
                only = "only ";
            }

            return Err(syn::Error::new(
                span,
                format!(
                    "format string requires {} arguments but {}{} were provided",
                    expected_arg_count, only, given_arg_count
                ),
            ));
        }

        let mut exprs = vec![];
        let mut patterns = vec![];

        for arg_index in 0..expected_arg_count {
            let arg_ident = format_ident!("arg{}", arg_index);
            let matching_param = params
                .iter()
                .find(|param| param.index == arg_index)
                .unwrap();

            let expr = encode_arg(&matching_param.ty, &params, arg_index, &arg_ident);

            exprs.push(expr);
            patterns.push(arg_ident);
        }

        Ok(Codegen { exprs, patterns })
    }
}

fn encode_arg(ty: &Type, params: &[Parameter], arg_index: usize, arg: &Ident2) -> TokenStream2 {
    match ty {
        Type::I8 => quote!(defmt::export::i8(#arg)),
        Type::I16 => quote!(defmt::export::i16(#arg)),
        Type::I32 => quote!(defmt::export::i32(#arg)),
        Type::I64 => quote!(defmt::export::i64(#arg)),
        Type::I128 => quote!(defmt::export::i128(#arg)),
        Type::Isize => quote!(defmt::export::isize(#arg)),

        Type::U8 => quote!(defmt::export::u8(#arg)),
        Type::U16 => quote!(defmt::export::u16(#arg)),
        Type::U32 => quote!(defmt::export::u32(#arg)),
        Type::U64 => quote!(defmt::export::u64(#arg)),
        Type::U128 => quote!(defmt::export::u128(#arg)),
        Type::Usize => quote!(defmt::export::usize(#arg)),

        Type::F32 => quote!(defmt::export::f32(#arg)),
        Type::F64 => quote!(defmt::export::f64(#arg)),

        Type::Bool => quote!(defmt::export::bool(#arg)),

        Type::Str => quote!(defmt::export::str(#arg)),
        Type::IStr => quote!(defmt::export::istr(#arg)),
        Type::Char => quote!(defmt::export::char(#arg)),

        Type::Format => quote!(defmt::export::fmt(#arg)),
        Type::FormatSlice => quote!(defmt::export::fmt_slice(#arg)),
        Type::FormatArray(len) => quote!(defmt::export::fmt_array({
            let tmp: &[_; #len] = #arg;
            tmp
        })),

        Type::Debug => quote!(defmt::export::debug(#arg)),
        Type::Display => quote!(defmt::export::display(#arg)),
        Type::FormatSequence => unreachable!(),

        Type::U8Slice => quote!(defmt::export::slice(#arg)),

        // We cast to the expected array type (which should be a no-op cast) to provoke
        // a type mismatch error on mismatched lengths:
        // ``Symbol’s value as variable is void: //
        Type::U8Array(len) => quote!(defmt::export::u8_array({
            let tmp: &[u8; #len] = #arg;
            tmp
        })),

        Type::BitField(_) => {
            let all_bitfields = params.iter().filter(|param| param.index == arg_index);
            let (smallest_bit_index, largest_bit_index) =
                defmt_parser::get_max_bitfield_range(all_bitfields).unwrap();

            // indices of the lowest and the highest octet which contains bitfield-relevant data
            let lowest_byte = smallest_bit_index / 8;
            let highest_byte = (largest_bit_index - 1) / 8;
            let truncated_sz = highest_byte - lowest_byte + 1; // in bytes

            // shift away unneeded lower octet
            // TODO: create helper for shifting because readability
            match truncated_sz {
                1 => {
                    quote!(defmt::export::u8(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))
                }
                2 => {
                    quote!(defmt::export::u16(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))
                }
                3..=4 => {
                    quote!(defmt::export::u32(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))
                }
                5..=8 => {
                    quote!(defmt::export::u64(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))
                }
                9..=16 => {
                    quote!(defmt::export::u128(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8))))
                }
                _ => unreachable!(),
            }
        }
    }
}
