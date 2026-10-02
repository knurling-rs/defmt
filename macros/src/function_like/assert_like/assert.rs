use defmt_parser::Level;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{construct, function_like::log};

pub(crate) fn expand(args: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as super::Args);

    let condition = args.condition;
    let (format_string, formatting_args) = if let Some(log_args) = args.log_args {
        let format_string = format!("panicked at '{}'", log_args.format_string.value());
        (format_string, log_args.formatting_args)
    } else {
        let format_string = format!(
            "panicked at 'assertion failed: {}'",
            construct::escaped_expr_string(&condition)
        );
        (format_string, None)
    };

    let format_string = construct::string_literal(&format_string);
    let log_and_panic = match log::expand_parsed_and_panic(
        Level::Error,
        log::Args {
            format_string,
            formatting_args,
        },
    ) {
        Ok(log_and_panic) => log_and_panic,
        Err(err) => return err.into_compile_error().into(),
    };

    quote!(
        if !(#condition) {
            #log_and_panic
        }
    )
    .into()
}
