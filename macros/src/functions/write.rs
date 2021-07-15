use defmt_parser::ParserMode;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::Codegen;

use self::args::Args;

mod args;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Args);
    let ls = args.format_string.value();
    let fragments = match defmt_parser::parse(&ls, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => {
            abort!(args.format_string, "{}", e)
        }
    };

    let format_exprs: Vec<_> = args
        .format_args
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or_default();

    let (pats, exprs) =
        match Codegen::new(&fragments, format_exprs.len(), args.format_string.span()) {
            Ok(cg) => (cg.pats, cg.exprs),
            Err(e) => return e.to_compile_error().into(),
        };

    let formatter = &args.formatter;
    let sym = crate::mksym(&ls, "write", false);
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
