use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, LitStr, Token,
};

pub(crate) struct Args {
    pub(crate) format_string: LitStr,
    pub(crate) formatting_args: Option<Punctuated<FormatArg, Token![,]>>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            format_string: input.parse()?,
            formatting_args: if input.is_empty() {
                None
            } else {
                let _comma: Token![,] = input.parse()?;
                Some(Punctuated::parse_terminated(input)?)
            },
        })
    }
}

/// A formatting argument: `expr` (positional) or `name = expr` (named).
pub(crate) enum FormatArg {
    Positional(Expr),
    Named(Ident, Expr),
}

impl From<Expr> for FormatArg {
    fn from(expr: Expr) -> Self {
        FormatArg::Positional(expr)
    }
}

impl Parse for FormatArg {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        // Like `core::format_args!`, a leading `ident =` (but not `ident ==`) denotes a
        // named argument.
        if input.peek(Ident) && input.peek2(Token![=]) && !input.peek2(Token![==]) {
            let name: Ident = input.parse()?;
            let _eq: Token![=] = input.parse()?;
            let expr: Expr = input.parse()?;
            Ok(FormatArg::Named(name, expr))
        } else {
            Ok(FormatArg::Positional(input.parse()?))
        }
    }
}
