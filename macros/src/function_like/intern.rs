use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

use crate::construct;

pub(crate) fn expand(args: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(args as LitStr);
    construct::interned_string(&literal.value(), "str", false).into()
}
