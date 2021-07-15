use defmt_parser::ParserMode;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;

use crate::{Codegen, FormatArgs};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FormatArgs);

    let format_string = input.litstr.value();

    let fragments = match defmt_parser::parse(&format_string, ParserMode::Strict) {
        Ok(args) => args,
        Err(e) => abort!(input.litstr, "{}", e),
    };

    let args: Vec<_> = input
        .rest
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or_default();

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), input.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let static_var_name = format_ident!("S");
    let static_var_item = crate::mkstatic(static_var_name.clone(), &format_string, "timestamp");

    quote!(
        const _: () = {
            #[export_name = "_defmt_timestamp"]
            fn defmt_timestamp(fmt: ::defmt::Formatter<'_>) {
                match (#(&(#args)),*) {
                    (#(#pats),*) => {
                    // NOTE: No format string index, and no finalize call.
                        #(#exprs;)*
                    }
                }
            }

            #static_var_item;

            // Unique symbol name to prevent multiple `timestamp!` invocations in the crate graph.
            // Uses `#static_var_name` to ensure it is not discarded by the linker.
            // This symbol itself is retained via a `EXTERN` directive in the linker script.
            #[no_mangle]
            #[cfg_attr(target_os = "macos", link_section = ".defmt,end.timestamp")]
            #[cfg_attr(not(target_os = "macos"), link_section = ".defmt.end.timestamp")]
            static __DEFMT_MARKER_TIMESTAMP_WAS_DEFINED: &u8 = &#static_var_name;
        };
    )
    .into()
}
