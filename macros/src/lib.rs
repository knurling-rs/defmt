//! INTERNAL; DO NOT USE. Please use the `defmt` crate to access the functionality implemented here

#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

use defmt_parser::Level;
use functions::assert_binop::BinOp;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;

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

/* Logging macros */
#[proc_macro_error]
#[proc_macro]
pub fn trace(input: TokenStream) -> TokenStream {
    functions::log::expand(Level::Trace, input)
}

#[proc_macro_error]
#[proc_macro]
pub fn debug(input: TokenStream) -> TokenStream {
    functions::log::expand(Level::Debug, input)
}

#[proc_macro_error]
#[proc_macro]
pub fn info(input: TokenStream) -> TokenStream {
    functions::log::expand(Level::Info, input)
}

#[proc_macro_error]
#[proc_macro]
pub fn warn(input: TokenStream) -> TokenStream {
    functions::log::expand(Level::Warn, input)
}

#[proc_macro_error]
#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    functions::log::expand(Level::Error, input)
}
/* Logging macros */

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
