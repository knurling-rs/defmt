use syn::{
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Expr, LitStr, Token,
};

pub(crate) struct Args {
    pub(crate) formatter: Expr,
    _comma: Token![,],
    pub(crate) format_string: LitStr,
    pub(crate) format_args: Option<(Token![,], Punctuated<Expr, Token![,]>)>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            formatter: input.parse()?,
            _comma: input.parse()?,
            format_string: input.parse()?,
            format_args: if input.is_empty() {
                None
            } else {
                Some((input.parse()?, Punctuated::parse_terminated(input)?))
            },
        })
    }
}
