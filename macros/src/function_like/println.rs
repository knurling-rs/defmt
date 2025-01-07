use defmt_parser::ParserMode;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error2::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::construct;
use crate::function_like::log::{Args, Codegen};

pub(crate) fn expand(args: TokenStream) -> TokenStream {
    expand_parsed(parse_macro_input!(args as Args)).into()
}

pub(crate) fn expand_parsed(args: Args) -> TokenStream2 {
    let format_string = args.format_string.value();
    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e @ defmt_parser::Error::UnknownDisplayHint(_)) => abort!(
            args.format_string, "{}", e;
            help = "`defmt` uses a slightly different syntax than regular formatting in Rust. See https://defmt.ferrous-systems.com/macros.html for more details.";
        ),
        Err(e) => abort!(args.format_string, "{}", e), // No extra help
    };

    let formatting_exprs = args
        .formatting_args
        .map(|punctuated| punctuated.into_iter().collect::<Vec<_>>())
        .unwrap_or_default();

    let Codegen { patterns, exprs } = Codegen::new(
        &fragments,
        formatting_exprs.len(),
        args.format_string.span(),
    );

    let header = construct::interned_string(&format_string, "println", true, None);
    let content = if exprs.is_empty() {
        quote!(
            defmt::export::acquire_header_and_release(&#header);
        )
    } else {
        quote!(
            // safety: will be released a few lines further down
            unsafe { defmt::export::acquire_and_header(&#header); };
            #(#exprs;)*
            // safety: acquire() was called a few lines above
            unsafe { defmt::export::release() }
        )
    };
    quote!({
        match (#(&(#formatting_exprs)),*) {
            (#(#patterns),*) => {
                #content
            }
        }
    })
}
