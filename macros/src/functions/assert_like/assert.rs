use defmt_parser::Level;
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, LitStr};

use crate::FormatArgs;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as super::Args);

    let condition = args.condition;
    let log_stmt = if let Some(args) = args.args {
        let panic_msg = format!("panicked at '{}'", args.litstr.value());
        let litstr = LitStr::new(&panic_msg, Span2::call_site());
        let rest = args.rest;

        crate::log(Level::Error, FormatArgs { litstr, rest })
    } else {
        let panic_msg = &format!(
            "panicked at 'assertion failed: {}'",
            crate::escape_expr(&condition)
        );
        let litstr = LitStr::new(panic_msg, Span2::call_site());

        crate::log(Level::Error, FormatArgs { litstr, rest: None })
    };

    quote!(
        if !(#condition) {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}
