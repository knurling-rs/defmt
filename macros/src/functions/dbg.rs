use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Token,
};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    codegen(&input)
}

struct Input {
    exprs: Punctuated<Expr, Token![,]>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Ok(Self {
            exprs: Punctuated::parse_terminated(input)?,
        })
    }
}

fn codegen(input: &Input) -> TokenStream {
    let tuple_exprs = input
        .exprs
        .iter()
        .map(|expr| {
            let escaped_expr = crate::escape_expr(&expr);
            let format_string = format!("{} = {{}}", escaped_expr);

            quote!(match #expr {
                tmp => {
                    defmt::trace!(#format_string, tmp);
                    tmp
                }
            })
        })
        .collect::<Vec<_>>();

    if tuple_exprs.is_empty() {
        // for compatibility with `std::dbg!` we also emit a TRACE log in this case
        quote!(defmt::trace!(""))
    } else {
        quote![ (#(#tuple_exprs),*) ]
    }
    .into()
}
