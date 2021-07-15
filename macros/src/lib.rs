//! INTERNAL; DO NOT USE. Please use the `defmt` crate to access the functionality implemented here

#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use defmt_parser::{Fragment, Level, ParserMode};
use functions::assert_binop::BinOp;
use proc_macro::TokenStream;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

mod attributes;
mod bitflags;
mod construct;
mod consts;
mod derives;
mod functions;
mod items;
mod symbol;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn global_logger(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::global_logger::expand(args, input)
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn panic_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::panic_handler::expand(args, input)
}

#[proc_macro_error]
#[proc_macro_derive(Format)]
pub fn format(input: TokenStream) -> TokenStream {
    derives::format::expand(input)
}

// not naming this `assert` to avoid shadowing `core::assert` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn assert_(input: TokenStream) -> TokenStream {
    functions::assert_like::assert::expand(input)
}

// not naming this `assert_eq` to avoid shadowing `core::assert_eq` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn assert_eq_(input: TokenStream) -> TokenStream {
    functions::assert_binop::expand(input, BinOp::Eq)
}

// not naming this `assert_ne` to avoid shadowing `core::assert_ne` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn assert_ne_(input: TokenStream) -> TokenStream {
    functions::assert_binop::expand(input, BinOp::Ne)
}

#[proc_macro_error]
#[proc_macro]
pub fn dbg(input: TokenStream) -> TokenStream {
    functions::dbg::expand(input)
}

// NOTE these `debug_*` macros can be written using `macro_rules!` (that'd be simpler) but that
// results in an incorrect source code location being reported: the location of the `macro_rules!`
// statement is reported. Using a proc-macro results in the call site being reported, which is what
// we want
#[proc_macro_error]
#[proc_macro]
pub fn debug_assert_(input: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_(input));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn debug_assert_eq_(input: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_eq_(input));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro]
#[proc_macro_error]
pub fn debug_assert_ne_(input: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_ne_(input));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro_error]
#[proc_macro]
pub fn intern(input: TokenStream) -> TokenStream {
    functions::intern::expand(input)
}

#[proc_macro_error]
#[proc_macro]
pub fn internp(input: TokenStream) -> TokenStream {
    functions::internp::expand(input)
}

// not naming this `panic` to avoid shadowing `core::panic` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn panic_(input: TokenStream) -> TokenStream {
    functions::panic_like::expand(input, "panicked at 'explicit panic'", |format_string| {
        format!("panicked at '{}'", format_string)
    })
}

// not naming this `todo` to avoid shadowing `core::todo` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn todo_(input: TokenStream) -> TokenStream {
    functions::panic_like::expand(
        input,
        "panicked at 'not yet implemented'",
        |format_string| format!("panicked at 'not yet implemented: {}'", format_string),
    )
}

// not naming this `unreachable` to avoid shadowing `core::unreachable` in this scope
#[proc_macro_error]
#[proc_macro]
pub fn unreachable_(input: TokenStream) -> TokenStream {
    functions::panic_like::expand(
        input,
        "panicked at 'internal error: entered unreachable code'",
        |format_string| {
            format!(
                "panicked at 'internal error: entered unreachable code: {}'",
                format_string
            )
        },
    )
}

#[proc_macro_error]
#[proc_macro]
pub fn unwrap(input: TokenStream) -> TokenStream {
    functions::assert_like::unwrap::expand(input)
}

#[proc_macro_error]
#[proc_macro]
pub fn write(input: TokenStream) -> TokenStream {
    functions::write::expand(input)
}

#[proc_macro_error]
#[proc_macro]
pub fn timestamp(input: TokenStream) -> TokenStream {
    items::timestamp::expand(input)
}

#[proc_macro]
pub fn bitflags(ts: TokenStream) -> TokenStream {
    bitflags::expand(ts)
}

/// Returns a list of features of which one has to be enabled for `level` to be active
///
/// * `debug_assertions == true` means that dev profile is enabled
/// * `"defmt-default"` is enabled for dev & release profile so debug_assertions does not matter
fn necessary_features_for_level(level: Level, debug_assertions: bool) -> &'static [&'static str] {
    match level {
        Level::Trace if debug_assertions => &["defmt-trace", "defmt-default"],
        Level::Debug if debug_assertions => &["defmt-debug", "defmt-trace", "defmt-default"],

        Level::Trace => &["defmt-trace"],
        Level::Debug => &["defmt-debug", "defmt-trace"],
        Level::Info => &["defmt-info", "defmt-debug", "defmt-trace", "defmt-default"],
        Level::Warn => &[
            "defmt-warn",
            "defmt-info",
            "defmt-debug",
            "defmt-trace",
            "defmt-default",
        ],
        Level::Error => &[
            "defmt-error",
            "defmt-warn",
            "defmt-info",
            "defmt-debug",
            "defmt-trace",
            "defmt-default",
        ],
    }
}

fn cfg_if_logging_enabled(level: Level) -> TokenStream2 {
    let features_dev = necessary_features_for_level(level, true);
    let features_release = necessary_features_for_level(level, false);

    quote!(
        any(
            all(    debug_assertions,  any(#( feature = #features_dev     ),*)),
            all(not(debug_assertions), any(#( feature = #features_release ),*))
        )
    )
}

fn log_ts(level: Level, ts: TokenStream) -> TokenStream {
    log(level, parse_macro_input!(ts as FormatArgs)).into()
}

fn log(level: Level, log: FormatArgs) -> TokenStream2 {
    let ls = log.litstr.value();
    let fragments = match defmt_parser::parse(&ls, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => return parse::Error::new(log.litstr.span(), e).to_compile_error(),
    };

    let args = log
        .rest
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or_else(Vec::new);

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), log.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error(),
    };

    let sym = mksym(&ls, level.as_str(), true);
    let logging_enabled = cfg_if_logging_enabled(level);
    quote!({
        #[cfg(#logging_enabled)] {
            match (#(&(#args)),*) {
                (#(#pats),*) => {
                    defmt::export::acquire();
                    defmt::export::header(&#sym);
                    #(#exprs;)*
                    defmt::export::release()
                }
            }
        }
        // if logging is disabled match args, so they are not unused
        #[cfg(not(#logging_enabled))]
        match (#(&(#args)),*) {
            _ => {}
        }
    })
}

#[proc_macro]
pub fn trace(ts: TokenStream) -> TokenStream {
    log_ts(Level::Trace, ts)
}

#[proc_macro]
pub fn debug(ts: TokenStream) -> TokenStream {
    log_ts(Level::Debug, ts)
}

#[proc_macro]
pub fn info(ts: TokenStream) -> TokenStream {
    log_ts(Level::Info, ts)
}

#[proc_macro]
pub fn warn(ts: TokenStream) -> TokenStream {
    log_ts(Level::Warn, ts)
}

#[proc_macro]
pub fn error(ts: TokenStream) -> TokenStream {
    log_ts(Level::Error, ts)
}

fn escape_expr(expr: &Expr) -> String {
    let q = quote!(#expr);
    q.to_string().replace("{", "{{").replace("}", "}}")
}

struct FormatArgs {
    litstr: LitStr,
    rest: Option<(Token![,], Punctuated<Expr, Token![,]>)>,
}

impl Parse for FormatArgs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            litstr: input.parse()?,
            rest: if input.is_empty() {
                None
            } else {
                Some((input.parse()?, Punctuated::parse_terminated(input)?))
            },
        })
    }
}

/// work around restrictions on length and allowed characters imposed by macos linker
/// returns (note the comma character for macos):
///   under macos: ".defmt," + 16 character hex digest of symbol's hash
///   otherwise:   ".defmt." + prefix + symbol
fn mksection(macos: bool, prefix: &str, symbol: &str) -> String {
    let mut sub_section = format!(".{}{}", prefix, symbol);

    if macos {
        let mut hasher = DefaultHasher::new();
        sub_section.hash(&mut hasher);
        sub_section = format!(",{:x}", hasher.finish());
    }

    format!(".defmt{}", sub_section)
}

fn mkstatic(varname: Ident2, string: &str, tag: &str) -> TokenStream2 {
    let sym = symbol::Symbol::new(tag, string).mangle();
    let section = mksection(false, "", &sym);
    let section_macos = mksection(true, "", &sym);

    quote!(
        #[cfg_attr(target_os = "macos", link_section = #section_macos)]
        #[cfg_attr(not(target_os = "macos"), link_section = #section)]
        #[export_name = #sym]
        static #varname: u8 = 0;
    )
}

fn mksym(string: &str, tag: &str, is_log_statement: bool) -> TokenStream2 {
    // NOTE we rely on this variable name when extracting file location information from the DWARF
    // without it we have no other mean to differentiate static variables produced by `info!` vs
    // produced by `intern!` (or `internp`)
    let varname = if is_log_statement {
        format_ident!("DEFMT_LOG_STATEMENT")
    } else {
        format_ident!("S")
    };
    let sym = if cfg!(feature = "unstable-test") {
        quote!({ defmt::export::fetch_add_string_index() })
    } else {
        let statik = mkstatic(varname.clone(), string, tag);
        quote!({
            #statik
            &#varname as *const u8 as u16
        })
    };

    quote!({
        defmt::export::make_istr(#sym)
    })
}

struct Codegen {
    pats: Vec<Ident2>,
    exprs: Vec<TokenStream2>,
}

impl Codegen {
    fn new(fragments: &[Fragment<'_>], num_args: usize, span: Span2) -> parse::Result<Self> {
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
                defmt_parser::Type::I8 => exprs.push(quote!(defmt::export::i8(#arg))),
                defmt_parser::Type::I16 => exprs.push(quote!(defmt::export::i16(#arg))),
                defmt_parser::Type::I32 => exprs.push(quote!(defmt::export::i32(#arg))),
                defmt_parser::Type::I64 => exprs.push(quote!(defmt::export::i64(#arg))),
                defmt_parser::Type::I128 => exprs.push(quote!(defmt::export::i128(#arg))),
                defmt_parser::Type::Isize => exprs.push(quote!(defmt::export::isize(#arg))),

                defmt_parser::Type::U8 => exprs.push(quote!(defmt::export::u8(#arg))),
                defmt_parser::Type::U16 => exprs.push(quote!(defmt::export::u16(#arg))),
                defmt_parser::Type::U32 => exprs.push(quote!(defmt::export::u32(#arg))),
                defmt_parser::Type::U64 => exprs.push(quote!(defmt::export::u64(#arg))),
                defmt_parser::Type::U128 => exprs.push(quote!(defmt::export::u128(#arg))),
                defmt_parser::Type::Usize => exprs.push(quote!(defmt::export::usize(#arg))),

                defmt_parser::Type::F32 => exprs.push(quote!(defmt::export::f32(#arg))),
                defmt_parser::Type::F64 => exprs.push(quote!(defmt::export::f64(#arg))),

                defmt_parser::Type::Bool => exprs.push(quote!(defmt::export::bool(#arg))),

                defmt_parser::Type::Str => exprs.push(quote!(defmt::export::str(#arg))),
                defmt_parser::Type::IStr => exprs.push(quote!(defmt::export::istr(#arg))),
                defmt_parser::Type::Char => exprs.push(quote!(defmt::export::char(#arg))),

                defmt_parser::Type::Format => exprs.push(quote!(defmt::export::fmt(#arg))),
                defmt_parser::Type::FormatSlice => {
                    exprs.push(quote!(defmt::export::fmt_slice(#arg)))
                }
                defmt_parser::Type::FormatArray(len) => {
                    exprs.push(quote!(defmt::export::fmt_array({
                        let tmp: &[_; #len] = #arg;
                        tmp
                    })))
                }

                defmt_parser::Type::Debug => exprs.push(quote!(defmt::export::debug(#arg))),
                defmt_parser::Type::Display => exprs.push(quote!(defmt::export::display(#arg))),
                defmt_parser::Type::FormatSequence => unreachable!(),

                defmt_parser::Type::U8Slice => exprs.push(quote!(defmt::export::slice(#arg))),
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
                defmt_parser::Type::U8Array(len) => exprs.push(quote!(defmt::export::u8_array({
                    let tmp: &[u8; #len] = #arg;
                    tmp
                }))),
                defmt_parser::Type::BitField(_) => {
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

            let message = format!(
                "format string requires {} arguments but {}{} were provided",
                actual_argument_count, only, num_args
            );
            return Err(parse::Error::new(span, message));
        }

        Ok(Codegen { pats, exprs })
    }
}
