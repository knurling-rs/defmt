use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

use crate::construct;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    construct::interned_string(&lit.value(), "str", false).into()
}
