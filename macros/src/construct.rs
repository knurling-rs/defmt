use proc_macro2::Span as Span2;
use syn::{parse_quote, Expr, Ident, LitStr};

pub(crate) fn string(content: &str) -> LitStr {
    LitStr::new(content, Span2::call_site())
}

pub(crate) fn variable(name: &str) -> Expr {
    let ident = Ident::new(name, Span2::call_site());
    parse_quote!(#ident)
}
