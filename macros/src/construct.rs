use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash as _, Hasher as _},
};

use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{parse_quote, Expr, Ident, LitStr};

use crate::symbol::Symbol;

pub(crate) fn interned_string(string: &str, tag: &str, is_log_statement: bool) -> TokenStream2 {
    // NOTE we rely on this variable name when extracting file location information from the DWARF
    // without it we have no other mean to differentiate static variables produced by `info!` vs
    // produced by `intern!` (or `internp`)
    let var_name = if is_log_statement {
        format_ident!("DEFMT_LOG_STATEMENT")
    } else {
        format_ident!("S")
    };

    let sym = if cfg!(feature = "unstable-test") {
        quote!({ defmt::export::fetch_add_string_index() })
    } else {
        let var_item = static_variable(&var_name, string, tag);
        quote!({
            #var_item
            &#var_name as *const u8 as u16
        })
    };

    quote!({
        defmt::export::make_istr(#sym)
    })
}

/// work around restrictions on length and allowed characters imposed by macos linker
/// returns (note the comma character for macos):
///   under macos: ".defmt," + 16 character hex digest of symbol's hash
///   otherwise:   ".defmt." + prefix + symbol
pub(crate) fn linker_section(for_macos: bool, prefix: Option<&str>, symbol: &str) -> String {
    let mut sub_section = if let Some(prefix) = prefix {
        format!(".{}.{}", prefix, symbol)
    } else {
        format!(".{}", symbol)
    };

    if for_macos {
        let mut hasher = DefaultHasher::new();
        sub_section.hash(&mut hasher);
        sub_section = format!(",{:x}", hasher.finish());
    }

    format!(".defmt{}", sub_section)
}

pub(crate) fn static_variable(name: &Ident2, data: &str, tag: &str) -> TokenStream2 {
    let sym = Symbol::new(tag, data).mangle();
    let section = linker_section(false, None, &sym);
    let section_macos = linker_section(true, None, &sym);

    quote!(
        #[cfg_attr(target_os = "macos", link_section = #section_macos)]
        #[cfg_attr(not(target_os = "macos"), link_section = #section)]
        #[export_name = #sym]
        static #name: u8 = 0;
    )
}

pub(crate) fn string(content: &str) -> LitStr {
    LitStr::new(content, Span2::call_site())
}

pub(crate) fn variable(name: &str) -> Expr {
    let ident = Ident::new(name, Span2::call_site());
    parse_quote!(#ident)
}
