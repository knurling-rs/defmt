use defmt_parser::{Level, ParserMode};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort;
use quote::quote;
use syn::parse_macro_input;

use crate::construct;

pub(crate) use self::{args::Args, codegen::Codegen};

mod args;
mod codegen;

pub(crate) fn expand(level: Level, args: TokenStream) -> TokenStream {
    expand_parsed(level, parse_macro_input!(args as Args)).into()
}

pub(crate) fn expand_parsed(level: Level, args: Args) -> TokenStream2 {
    let format_string = args.format_string.value();
    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => abort!(args.format_string, "{}", e),
    };

    let formatting_exprs = args
        .formatting_args
        .map(|punctuated| punctuated.into_iter().collect())
        .unwrap_or_else(Vec::new);

    let Codegen { patterns, exprs } = Codegen::new(
        &fragments,
        formatting_exprs.len(),
        args.format_string.span(),
    );

    let header = construct::interned_string(&format_string, level.as_str(), true);
    let logging_enabled = cfg_if_logging_enabled(level);
    quote!({
        #[cfg(#logging_enabled)] {
            match (#(&(#formatting_exprs)),*) {
                (#(#patterns),*) => {
                    defmt::export::acquire();
                    defmt::export::header(&#header);
                    #(#exprs;)*
                    defmt::export::release()
                }
            }
        }
        // if logging is disabled match args, so they are not unused
        #[cfg(not(#logging_enabled))]
        match (#(&(#formatting_exprs)),*) {
            _ => {}
        }
    })
}

fn cfg_if_logging_enabled(level: Level) -> TokenStream2 {
    let features_dev = necessary_features_for_level(level, true);
    let features_release = necessary_features_for_level(level, false);

    quote!(
        any(
            all(    debug_assertions,  any(#( feature = #features_dev     ),*)),
            all(not(debug_assertions), any(#( feature = #features_release ),*))
        )
    )
}

/// Returns a list of features of which one has to be enabled for `level` to be active
///
/// * `debug_assertions == true` means that dev profile is enabled
/// * `"defmt-default"` is enabled for dev & release profile so debug_assertions does not matter
fn necessary_features_for_level(level: Level, debug_assertions: bool) -> &'static [&'static str] {
    match level {
        Level::Trace if debug_assertions => &["defmt-trace", "defmt-default"],
        Level::Debug if debug_assertions => &["defmt-debug", "defmt-trace", "defmt-default"],

        Level::Trace => &["defmt-trace"],
        Level::Debug => &["defmt-debug", "defmt-trace"],
        Level::Info => &["defmt-info", "defmt-debug", "defmt-trace", "defmt-default"],
        Level::Warn => &[
            "defmt-warn",
            "defmt-info",
            "defmt-debug",
            "defmt-trace",
            "defmt-default",
        ],
        Level::Error => &[
            "defmt-error",
            "defmt-warn",
            "defmt-info",
            "defmt-debug",
            "defmt-trace",
            "defmt-default",
        ],
    }
}
