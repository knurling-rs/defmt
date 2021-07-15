use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    crate::mksym(&lit.value(), "str", false).into()
}
