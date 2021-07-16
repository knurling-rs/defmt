use defmt_parser::Level;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{construct, function_like::log};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as super::Args);

    let condition = args.condition;
    let (format_string, formatting_args) = if let Some(log_args) = args.log_args {
        let panic_msg = format!("panicked at '{}'", log_args.format_string.value());

        (
            construct::string_literal(&panic_msg),
            log_args.formatting_args,
        )
    } else {
        let panic_msg = &format!(
            "panicked at 'assertion failed: {}'",
            construct::escaped_expr_string(&condition)
        );

        (construct::string_literal(panic_msg), None)
    };

    let log_stmt = log::expand_parsed(
        Level::Error,
        log::Args {
            format_string,
            formatting_args,
        },
    );

    quote!(
        if !(#condition) {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}
