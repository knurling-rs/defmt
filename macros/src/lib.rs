use core::fmt::Write as _;
use proc_macro::TokenStream;

use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Data, DeriveInput, Expr, Fields, FieldsNamed, FieldsUnnamed, LitInt, LitStr, Token, Type,
};

// `#[derive(Format)]`
#[proc_macro_derive(Format)]
pub fn format(ts: TokenStream) -> TokenStream {
    let input = parse_macro_input!(ts as DeriveInput);
    let span = input.span();

    let ident = input.ident;
    let mut fs = String::new();
    let mut exprs = vec![];
    match input.data {
        Data::Enum(de) => {
            if de.variants.len() > 256 {
                return parse::Error::new(
                    span,
                    "`#[derive(Format)]` does not support enums with more than 256 variants",
                )
                .to_compile_error()
                .into();
            }

            let mut arms = vec![];
            let mut first = true;
            for (var, i) in de.variants.iter().zip(0u8..) {
                let vident = &var.ident;

                if first {
                    first = false;
                } else {
                    fs.push('|');
                }
                fs.push_str(&vident.to_string());

                let mut pats = quote!();
                let exprs = fields(
                    &var.fields,
                    &mut fs,
                    Kind::Enum {
                        patterns: &mut pats,
                    },
                );

                arms.push(quote!(
                    #ident::#vident #pats => {
                        f.u8(&#i);
                        #(#exprs)*
                    }
                ))
            }

            let id = rand::random::<u64>();
            let section = format!(".binfmt.fmt.{}", id);
            let sym = format!("{}@{}", fs, id);
            exprs.push(quote!(
                #[link_section = #section]
                #[export_name = #sym]
                static S: u8 = 0;

                f.str(&binfmt::export::str(&S));
            ));
            exprs.push(quote!(match self {
                #(#arms)*
            }));
        }

        Data::Struct(ds) => {
            fs = ident.to_string();
            let args = fields(&ds.fields, &mut fs, Kind::Struct);
            exprs.push(quote!(binfmt::write!(f, #fs #(,#args)*);))
        }

        Data::Union(..) => {
            return parse::Error::new(span, "`#[derive(Format)]` does not support unions")
                .to_compile_error()
                .into();
        }
    }

    quote!(
        impl binfmt::Format for #ident {
            fn format(&self, f: &mut binfmt::Formatter) {
                #(#exprs)*
            }
        }
    )
    .into()
}

enum Kind<'p> {
    Struct,
    Enum { patterns: &'p mut TokenStream2 },
}

fn fields(fields: &Fields, format: &mut String, mut kind: Kind) -> Vec<TokenStream2> {
    let mut list = vec![];
    match fields {
        Fields::Named(FieldsNamed { named: fs, .. })
        | Fields::Unnamed(FieldsUnnamed { unnamed: fs, .. }) => {
            let named = match fields {
                Fields::Named(..) => true,
                Fields::Unnamed(..) => false,
                _ => unreachable!(),
            };

            if !fs.is_empty() {
                if named {
                    format.push_str(" {{ ");
                } else {
                    format.push_str("(");
                }
                let mut first = true;
                let mut pats = vec![];
                for (i, f) in fs.iter().enumerate() {
                    if first {
                        first = false;
                    } else {
                        format.push_str(", ");
                    }
                    let ty = match &f.ty {
                        Type::Path(p) => {
                            if let Some(ident) = p.path.get_ident() {
                                if ident == "u8" {
                                    "u8"
                                } else if ident == "u16" {
                                    "u16"
                                } else if ident == "u32" {
                                    "u32"
                                } else if ident == "i8" {
                                    "i8"
                                } else if ident == "i16" {
                                    "i16"
                                } else if ident == "i32" {
                                    "i32"
                                } else {
                                    "?"
                                }
                            } else {
                                "?"
                            }
                        }
                        _ => "?",
                    };
                    if let Some(ident) = f.ident.as_ref() {
                        core::write!(format, "{}: {{:{}}}", ident, ty).ok();

                        match &kind {
                            Kind::Struct => {
                                list.push(quote!(self.#ident));
                            }
                            Kind::Enum { .. } => {
                                let method = if ty == "?" {
                                    format_ident!("format")
                                } else {
                                    format_ident!("{}", ty)
                                };
                                list.push(quote!(f.#method(#ident)));
                                pats.push(ident.clone());
                            }
                        }
                    } else {
                        core::write!(format, "{{:{}}}", ty).ok();

                        match &kind {
                            Kind::Struct => {
                                let ident = LitInt::new(&i.to_string(), Span2::call_site());
                                list.push(quote!(self.#ident));
                            }
                            Kind::Enum { .. } => {
                                let ident = format_ident!("arg{}", i);
                                let method = if ty == "?" {
                                    format_ident!("format")
                                } else {
                                    format_ident!("{}", ty)
                                };
                                list.push(quote!(f.#method(#ident)));
                                pats.push(ident);
                            }
                        }
                    }
                }
                if named {
                    format.push_str(" }}");
                } else {
                    format.push_str(")");
                }

                if let Kind::Enum { patterns } = &mut kind {
                    if named {
                        **patterns = quote!({ #(#pats)* })
                    } else {
                        **patterns = quote!((#(#pats)*))
                    }
                }
            }
        }

        Fields::Unit => {}
    }

    list
}

#[proc_macro]
pub fn info(ts: TokenStream) -> TokenStream {
    let log = parse_macro_input!(ts as Log);
    let ls = log.litstr.value();
    let params = match Param::parse(&ls) {
        Ok(args) => args,
        Err(e) => {
            return parse::Error::new(log.litstr.span(), e)
                .to_compile_error()
                .into()
        }
    };

    let args = log
        .rest
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or(vec![]);

    let (pats, exprs) = match Codegen::new(&params, args.len(), log.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let id = rand::random::<u64>();
    let section = format!(".binfmt.info.{}", id);
    let sym = format!("{}@{}", ls, id);
    quote!({
        if cfg!(feature = "binfmt") {
            if binfmt::export::Level::Info >= binfmt::export::threshold() {
                if let Some(mut _fmt_) = binfmt::export::acquire() {
                    match (binfmt::export::timestamp(), #(#args),*) {
                        (ts, #(ref #pats),*) => {
                            #[link_section = #section]
                            #[export_name = #sym]
                            static S: u8 = 0;

                            _fmt_.str(&binfmt::export::str(&S));
                            _fmt_.leb64(ts);
                            #(#exprs;)*
                            binfmt::export::release(_fmt_)
                        }
                    }
                }
            }
        }
    })
    .into()
}

struct Log {
    litstr: LitStr,
    rest: Option<(Token![,], Punctuated<Expr, Token![,]>)>,
}

impl Parse for Log {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            litstr: input.parse()?,
            rest: if input.is_empty() {
                None
            } else {
                Some((input.parse()?, Punctuated::parse_separated_nonempty(input)?))
            },
        })
    }
}

#[proc_macro]
pub fn intern(ts: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(ts as LitStr);
    let ls = lit.value();
    if ls.contains('@') {
        return parse::Error::new(
            ls.span(),
            "strings that contain the character `@` cannot be interned",
        )
        .to_compile_error()
        .into();
    }

    let id = rand::random::<u64>();
    let section = format!(".binfmt.str.{}", id);
    let sym = format!("{}@{}", ls, id);
    quote!({
        #[link_section = #section]
        #[export_name = #sym]
        static S: u8 = 0;
        binfmt::export::str(&S)
    })
    .into()
}

#[proc_macro]
pub fn internp(ts: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(ts as LitStr);
    let ls = lit.value();
    if ls.contains('@') {
        return parse::Error::new(
            ls.span(),
            "strings that contain the character `@` cannot be interned",
        )
        .to_compile_error()
        .into();
    }

    // NOTE(no random id) these won't collide because they are limited in use
    let section = format!(".binfmt.prim.{}", ls);
    let sym = ls;
    quote!({
        #[link_section = #section]
        #[export_name = #sym]
        static S: u8 = 0;
        &S as *const u8 as u8
    })
    .into()
}

#[proc_macro]
pub fn write(ts: TokenStream) -> TokenStream {
    let write = parse_macro_input!(ts as Write);
    let ls = write.litstr.value();
    let params = match Param::parse(&ls) {
        Ok(args) => args,
        Err(e) => {
            return parse::Error::new(write.litstr.span(), e)
                .to_compile_error()
                .into()
        }
    };

    let args = write
        .rest
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or(vec![]);

    let (pats, exprs) = match Codegen::new(&params, args.len(), write.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let fmt = &write.fmt;
    let id = rand::random::<u64>();
    let section = format!(".binfmt.fmt.{}", id);
    let sym = format!("{}@{}", ls, id);
    quote!(match (#fmt, #(#args),*) {
        (ref mut _fmt_, #(ref #pats),*) => {
            #[link_section = #section]
            #[export_name = #sym]
            static S: u8 = 0;

            _fmt_.str(&binfmt::export::str(&S));
            #(#exprs;)*
        }
    })
    .into()
}

struct Write {
    fmt: Expr,
    _comma: Token![,],
    litstr: LitStr,
    rest: Option<(Token![,], Punctuated<Expr, Token![,]>)>,
}

impl Parse for Write {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self {
            fmt: input.parse()?,
            _comma: input.parse()?,
            litstr: input.parse()?,
            rest: if input.is_empty() {
                None
            } else {
                Some((input.parse()?, Punctuated::parse_separated_nonempty(input)?))
            },
        })
    }
}

struct Codegen {
    pats: Vec<Ident2>,
    exprs: Vec<TokenStream2>,
}

impl Codegen {
    fn new(params: &[Param], nargs: usize, span: Span2) -> parse::Result<Self> {
        let mut exprs = vec![];
        let mut pats = vec![];
        let mut n = 0;
        for param in params {
            match param {
                Param::Fmt => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.fmt(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::I16 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.i16(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::I32 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.i32(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::I8 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.i8(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::Str => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.str(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::U16 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.u16(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::U24 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.u24(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::U32 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.u32(#arg)));
                    pats.push(arg);
                    n += 1;
                }

                Param::U8 => {
                    let arg = format_ident!("arg{}", n);
                    exprs.push(quote!(_fmt_.u8(#arg)));
                    pats.push(arg);
                    n += 1;
                }
            }
        }

        if nargs < n {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but only {} were provided",
                    n, nargs
                ),
            ));
        }

        if nargs > n {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but {} were provided",
                    n, nargs
                ),
            ));
        }

        Ok(Codegen { pats, exprs })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Param {
    Fmt, // "{:?}"
    I16,
    I32,
    I8,
    Str,
    U16,
    U24,
    U32,
    U8,
}

impl Param {
    fn parse(s: &str) -> Result<Vec<Param>, &'static str> {
        static EOF: &str = "expected `}` but string was terminated";

        let mut chars = s.chars();

        let mut args = vec![];
        while let Some(c) = chars.next() {
            match c {
                '{' => {
                    match chars.next() {
                        // escaped `{`
                        Some('{') => {}

                        // format argument
                        Some(':') => {
                            static FMT: &str = "?}";
                            static STR: &str = "str}";
                            static U8: &str = "u8}";
                            static U16: &str = "u16}";
                            static U24: &str = "u24}";
                            static U32: &str = "u32}";
                            static I8: &str = "i8}";
                            static I16: &str = "i16}";
                            static I32: &str = "i32}";

                            let s = chars.as_str();
                            if s.starts_with(FMT) {
                                (0..FMT.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::Fmt);
                            } else if s.starts_with(STR) {
                                (0..STR.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::Str);
                            } else if s.starts_with(U8) {
                                (0..U8.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::U8);
                            } else if s.starts_with(U16) {
                                (0..U16.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::U16);
                            } else if s.starts_with(U24) {
                                (0..U24.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::U24);
                            } else if s.starts_with(U32) {
                                (0..U32.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::U32);
                            } else if s.starts_with(I8) {
                                (0..I8.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::I8);
                            } else if s.starts_with(I16) {
                                (0..I16.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::I16);
                            } else if s.starts_with(I32) {
                                (0..I32.len()).for_each(|_| drop(chars.next()));
                                args.push(Param::I32);
                            } else {
                                return Err("unknown format specifier");
                            }
                        }
                        Some(_) => return Err("`{` must be followed by `:`"),
                        None => return Err(EOF),
                    }
                }

                '}' => {
                    // must be a escaped `}`
                    if chars.next() != Some('}') {
                        return Err("unmatched `}` in format string");
                    }
                }

                '@' => return Err("format string cannot contain the `@` character"),

                _ => {}
            }
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::Param;

    #[test]
    fn args() {
        assert_eq!(Param::parse("{:?}"), Ok(vec![Param::Fmt]));
        assert_eq!(Param::parse("{:i16}"), Ok(vec![Param::I16]));
        assert_eq!(Param::parse("{:i32}"), Ok(vec![Param::I32]));
        assert_eq!(Param::parse("{:i8}"), Ok(vec![Param::I8]));
        assert_eq!(Param::parse("{:str}"), Ok(vec![Param::Str]));
        assert_eq!(Param::parse("{:u16}"), Ok(vec![Param::U16]));
        assert_eq!(Param::parse("{:u24}"), Ok(vec![Param::U24]));
        assert_eq!(Param::parse("{:u32}"), Ok(vec![Param::U32]));
        assert_eq!(Param::parse("{:u8}"), Ok(vec![Param::U8]));
    }
}
