use proc_macro2::Span as Span2;
use syn::LitStr;

pub(crate) fn string(content: &str) -> LitStr {
    LitStr::new(content, Span2::call_site())
}
