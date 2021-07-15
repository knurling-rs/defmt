use defmt_parser::Level;
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, LitStr};

use crate::FormatArgs;

pub(crate) fn expand(
    input: TokenStream,
    zero_args_string: &str,
    string_transform: impl FnOnce(&str) -> String,
) -> TokenStream {
    let log_stmt = if input.is_empty() {
        // panic!() -> error!("panicked at 'explicit panic'")
        let litstr = LitStr::new(zero_args_string, Span2::call_site());
        crate::log(Level::Error, FormatArgs { litstr, rest: None })
    } else {
        // panic!("a", b, c) -> error!("panicked at 'a'", b, c)
        let args = parse_macro_input!(input as FormatArgs);
        let litstr = LitStr::new(&string_transform(&args.litstr.value()), Span2::call_site());
        let rest = args.rest;
        crate::log(Level::Error, FormatArgs { litstr, rest })
    };

    quote!(
        {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}
