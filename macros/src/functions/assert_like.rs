use syn::{
    parse::{self, Parse, ParseStream},
    Expr, Token,
};

use crate::FormatArgs;

pub(crate) mod assert;
pub(crate) mod unwrap;

struct Args {
    args: Option<FormatArgs>,
    condition: Expr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let condition = input.parse()?;
        if input.is_empty() {
            // assert!(a)
            return Ok(Args {
                args: None,
                condition,
            });
        }

        let _comma: Token![,] = input.parse()?;

        if input.is_empty() {
            // assert!(a,)
            Ok(Args {
                args: None,
                condition,
            })
        } else {
            // assert!(a, "b", c)
            Ok(Args {
                args: Some(input.parse()?),
                condition,
            })
        }
    }
}
