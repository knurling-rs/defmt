use defmt_parser::Level;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{construct, function_like::log};

pub(crate) fn expand(
    input: TokenStream,
    zero_args_string: &str,
    string_transform: impl FnOnce(&str) -> String,
) -> TokenStream {
    let (format_string, formatting_args) = if input.is_empty() {
        // panic!() -> error!("panicked at 'explicit panic'")
        (construct::string(zero_args_string), None)
    } else {
        // panic!("a", b, c) -> error!("panicked at 'a'", b, c)
        let args = parse_macro_input!(input as log::Args);
        let format_string = construct::string(&string_transform(&args.format_string.value()));

        (format_string, args.formatting_args)
    };

    let log_stmt = log::expand_parsed(
        Level::Error,
        log::Args {
            format_string,
            formatting_args,
        },
    );

    quote!(
        {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}
