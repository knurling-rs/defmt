use defmt_parser::Level;
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, LitStr};

use crate::FormatArgs;

pub(crate) fn expand(ts: TokenStream) -> TokenStream {
    let args = parse_macro_input!(ts as super::Args);

    let condition = args.condition;
    let log_stmt = if let Some(args) = args.args {
        let panic_msg = format!("panicked at '{}'", args.litstr.value());
        let litstr = LitStr::new(&panic_msg, Span2::call_site());
        let rest = args.rest;

        crate::log(Level::Error, FormatArgs { litstr, rest })
    } else {
        let mut log_args = Punctuated::new();
        log_args.push(crate::ident_expr("_unwrap_err"));

        let litstr = LitStr::new(
            &format!(
                "panicked at 'unwrap failed: {}'\nerror: `{{:?}}`",
                crate::escape_expr(&condition)
            ),
            Span2::call_site(),
        );
        let rest = Some((Comma::default(), log_args));
        crate::log(Level::Error, FormatArgs { litstr, rest })
    };

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
