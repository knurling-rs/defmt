use defmt_parser::Level;
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Expr, LitStr};

use crate::FormatArgs;

mod parse;

pub(crate) fn expand(input: TokenStream, binop: BinOp) -> TokenStream {
    let args = parse_macro_input!(input as Args);

    let left = args.left;
    let right = args.right;

    let mut log_args = Punctuated::new();

    let extra_string = if let Some(args) = args.format_args {
        if let Some(rest) = args.rest {
            log_args.extend(rest.1);
        }
        format!(": {}", args.litstr.value())
    } else {
        String::new()
    };

    let vals = match binop {
        BinOp::Eq => &["left_val", "right_val"][..],
        BinOp::Ne => &["left_val"][..],
    };

    for val in vals {
        log_args.push(crate::ident_expr(*val));
    }

    let panic_msg = match binop {
        BinOp::Eq => format!(
            "panicked at 'assertion failed: `(left == right)`{}'
 left: `{{:?}}`
right: `{{:?}}`",
            extra_string
        ),
        BinOp::Ne => format!(
            "panicked at 'assertion failed: `(left != right)`{}'
left/right: `{{:?}}`",
            extra_string
        ),
    };

    let format_args = FormatArgs {
        litstr: LitStr::new(&panic_msg, Span2::call_site()),
        rest: Some((Comma::default(), log_args)),
    };
    let log_stmt = crate::log(Level::Error, format_args);

    let mut cond = quote!(*left_val == *right_val);
    if binop == BinOp::Eq {
        cond = quote!(!(#cond));
    }

    quote!(
        // evaluate arguments first
        match (&(#left), &(#right)) {
            (left_val, right_val) => {
                // following `core::assert_eq!`
                if #cond {
                    #log_stmt;
                    defmt::export::panic()
                }
            }
        }
    )
    .into()
}

#[derive(PartialEq)]
pub(crate) enum BinOp {
    Eq,
    Ne,
}

struct Args {
    left: Expr,
    right: Expr,
    format_args: Option<FormatArgs>,
}
