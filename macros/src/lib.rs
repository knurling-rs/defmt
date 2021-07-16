//! INTERNAL; DO NOT USE. Please use the `defmt` crate to access the functionality implemented here

#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

use defmt_parser::Level;
use function_like::assert_binop::BinOp;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;

mod attributes;
mod bitflags;
mod construct;
mod consts;
mod derives;
mod function_like;
mod items;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn global_logger(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::global_logger::expand(args, input)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn panic_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    attributes::panic_handler::expand(args, input)
}

#[proc_macro_derive(Format)]
#[proc_macro_error]
pub fn format(input: TokenStream) -> TokenStream {
    derives::format::expand(input)
}

// not naming this `assert` to avoid shadowing `core::assert` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn assert_(input: TokenStream) -> TokenStream {
    function_like::assert_like::assert::expand(input)
}

// not naming this `assert_eq` to avoid shadowing `core::assert_eq` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn assert_eq_(input: TokenStream) -> TokenStream {
    function_like::assert_binop::expand(input, BinOp::Eq)
}

// not naming this `assert_ne` to avoid shadowing `core::assert_ne` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn assert_ne_(input: TokenStream) -> TokenStream {
    function_like::assert_binop::expand(input, BinOp::Ne)
}

#[proc_macro]
#[proc_macro_error]
pub fn dbg(input: TokenStream) -> TokenStream {
    function_like::dbg::expand(input)
}

// NOTE these `debug_*` macros can be written using `macro_rules!` (that'd be simpler) but that
// results in an incorrect source code location being reported: the location of the `macro_rules!`
// statement is reported. Using a proc-macro results in the call site being reported, which is what
// we want
#[proc_macro]
#[proc_macro_error]
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

#[proc_macro]
#[proc_macro_error]
pub fn intern(input: TokenStream) -> TokenStream {
    function_like::intern::expand(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn internp(input: TokenStream) -> TokenStream {
    function_like::internp::expand(input)
}

/* Logging macros */
#[proc_macro]
#[proc_macro_error]
pub fn trace(input: TokenStream) -> TokenStream {
    function_like::log::expand(Level::Trace, input)
}

#[proc_macro]
#[proc_macro_error]
pub fn debug(input: TokenStream) -> TokenStream {
    function_like::log::expand(Level::Debug, input)
}

#[proc_macro]
#[proc_macro_error]
pub fn info(input: TokenStream) -> TokenStream {
    function_like::log::expand(Level::Info, input)
}

#[proc_macro]
#[proc_macro_error]
pub fn warn(input: TokenStream) -> TokenStream {
    function_like::log::expand(Level::Warn, input)
}

#[proc_macro]
#[proc_macro_error]
pub fn error(input: TokenStream) -> TokenStream {
    function_like::log::expand(Level::Error, input)
}
/* Logging macros */

// not naming this `panic` to avoid shadowing `core::panic` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn panic_(input: TokenStream) -> TokenStream {
    function_like::panic_like::expand(input, "panicked at 'explicit panic'", |format_string| {
        format!("panicked at '{}'", format_string)
    })
}

// not naming this `todo` to avoid shadowing `core::todo` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn todo_(input: TokenStream) -> TokenStream {
    function_like::panic_like::expand(
        input,
        "panicked at 'not yet implemented'",
        |format_string| format!("panicked at 'not yet implemented: {}'", format_string),
    )
}

// not naming this `unreachable` to avoid shadowing `core::unreachable` in this scope
#[proc_macro]
#[proc_macro_error]
pub fn unreachable_(input: TokenStream) -> TokenStream {
    function_like::panic_like::expand(
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

#[proc_macro]
#[proc_macro_error]
pub fn unwrap(input: TokenStream) -> TokenStream {
    function_like::assert_like::unwrap::expand(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn write(input: TokenStream) -> TokenStream {
    function_like::write::expand(input)
}

#[proc_macro]
#[proc_macro_error]
pub fn timestamp(input: TokenStream) -> TokenStream {
    items::timestamp::expand(input)
}

#[proc_macro]
pub fn bitflags(ts: TokenStream) -> TokenStream {
    bitflags::expand(ts)
}
