use defmt_parser::ParserMode;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::{construct, functions::log::Codegen};

use self::args::Args;

mod args;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let format_string = args.format_string.value();
    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => {
            abort!(args.format_string, "{}", e)
        }
    };

    let format_exprs: Vec<_> = args
        .format_args
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or_default();

    let Codegen { pats, exprs } =
        Codegen::new(&fragments, format_exprs.len(), args.format_string.span());

    let formatter = &args.formatter;
    let sym = construct::interned_string(&format_string, "write", false);
    quote!({
        let fmt: defmt::Formatter<'_> = #formatter;
        match (#(&(#format_exprs)),*) {
            (#(#pats),*) => {
                defmt::export::istr(&#sym);
                #(#exprs;)*
            }
        }
    })
    .into()
}
