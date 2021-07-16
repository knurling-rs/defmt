use defmt_parser::Level;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated};

use crate::{construct, function_like::log};

pub(crate) fn expand(ts: TokenStream) -> TokenStream {
    let args = parse_macro_input!(ts as super::Args);

    let condition = args.condition;
    let (format_string, formatting_args) = if let Some(log_args) = args.log_args {
        let panic_msg = format!("panicked at '{}'", log_args.format_string.value());

        (construct::string(&panic_msg), log_args.formatting_args)
    } else {
        let panic_msg = format!(
            "panicked at 'unwrap failed: {}'\nerror: `{{:?}}`",
            construct::escaped_expr_string(&condition)
        );

        let mut formatting_args = Punctuated::new();
        formatting_args.push(construct::variable("_unwrap_err"));

        (construct::string(&panic_msg), Some(formatting_args))
    };

    let log_stmt = log::expand_parsed(
        Level::Error,
        log::Args {
            format_string,
            formatting_args,
        },
    );

    quote!(
        match defmt::export::into_result(#condition) {
            ::core::result::Result::Ok(res) => res,
            ::core::result::Result::Err(_unwrap_err) => {
                #log_stmt;
                defmt::export::panic()
            }
        }
    )
    .into()
}
