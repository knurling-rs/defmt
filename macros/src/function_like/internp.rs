use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

use crate::{construct, symbol::Symbol};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let sym = Symbol::new("prim", &lit.value()).mangle();

    let prefix = Some("prim");
    let section = construct::linker_section(false, prefix, &sym);
    let section_for_macos = construct::linker_section(true, prefix, &sym);

    let sym = if cfg!(feature = "unstable-test") {
        quote!({ defmt::export::fetch_add_string_index() as u16 })
    } else {
        quote!({
            #[cfg_attr(target_os = "macos", link_section = #section_for_macos)]
            #[cfg_attr(not(target_os = "macos"), link_section = #section)]
            #[export_name = #sym]
            static S: u8 = 0;
            &S as *const u8 as u16
        })
    };

    quote!({
        defmt::export::make_istr(#sym)
    })
    .into()
}
