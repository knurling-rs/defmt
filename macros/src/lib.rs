use core::fmt::Write as _;
use proc_macro::{Span, TokenStream};

use binfmt_parser::Fragment;
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Data, DeriveInput, Expr, Fields, FieldsNamed, FieldsUnnamed, ItemFn, ItemStruct, LitInt,
    LitStr, ReturnType, Token, Type,
};

#[proc_macro_attribute]
pub fn global_logger(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(
            Span2::call_site(),
            "`#[global_logger]` attribute takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let s = parse_macro_input!(input as ItemStruct);
    let ident = &s.ident;
    let is_unit = match s.fields {
        Fields::Unit => true,
        _ => false,
    };
    if !s.generics.params.is_empty() || s.generics.where_clause.is_some() || !is_unit {
        return parse::Error::new(
            ident.span(),
            "struct must be a non-generic unit struct (e.g. `struct S;`)",
        )
        .to_compile_error()
        .into();
    }

    let vis = &s.vis;
    quote!(
        #vis struct #ident;

        #[no_mangle]
        unsafe fn _binfmt_acquire() -> Option<binfmt::Formatter> {
            <#ident as binfmt::Logger>::acquire().map(|nn| binfmt::Formatter::from_raw(nn))
        }

        #[no_mangle]
        unsafe fn _binfmt_release(f: binfmt::Formatter)  {
            <#ident as binfmt::Logger>::release(f.into_raw())
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn timestamp(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(
            Span2::call_site(),
            "`#[timestamp]` attribute takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let f = parse_macro_input!(input as ItemFn);

    let rety_is_ok = match &f.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(tp) => tp.path.get_ident().map(|id| id == "u64").unwrap_or(false),
            _ => false,
        },
    };

    let ident = &f.sig.ident;
    if f.sig.constness.is_some()
        || f.sig.asyncness.is_some()
        || f.sig.unsafety.is_some()
        || f.sig.abi.is_some()
        || !f.sig.generics.params.is_empty()
        || f.sig.generics.where_clause.is_some()
        || f.sig.variadic.is_some()
        || !f.sig.inputs.is_empty()
        || !rety_is_ok
    {
        return parse::Error::new(ident.span(), "function must have signature `fn() -> u64`")
            .to_compile_error()
            .into();
    }

    let block = &f.block;
    quote!(
        #[export_name = "_binfmt_timestamp"]
        fn #ident() -> u64 {
            #block
        }
    )
    .into()
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum MLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl MLevel {
    fn as_str(self) -> &'static str {
        match self {
            MLevel::Trace => "trace",
            MLevel::Debug => "debug",
            MLevel::Info => "info",
            MLevel::Warn => "warn",
            MLevel::Error => "error",
        }
    }

    // returns a list of features of which one has to be enabled for this Level to be active
    fn necessary_features(self, debug_assertions: bool) -> &'static [&'static str] {
        match self {
            MLevel::Trace => {
                if debug_assertions {
                    // dev profile
                    &["binfmt-trace", "binfmt-default"]
                } else {
                    &["binfmt-trace"]
                }
            }
            MLevel::Debug => {
                if debug_assertions {
                    // dev profile
                    &["binfmt-debug", "binfmt-trace", "binfmt-default"]
                } else {
                    &["binfmt-debug", "binfmt-trace"]
                }
            }
            MLevel::Info => {
                // binfmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &[
                    "binfmt-info",
                    "binfmt-debug",
                    "binfmt-trace",
                    "binfmt-default",
                ]
            }
            MLevel::Warn => {
                // binfmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &[
                    "binfmt-warn",
                    "binfmt-info",
                    "binfmt-debug",
                    "binfmt-trace",
                    "binfmt-default",
                ]
            }
            MLevel::Error => {
                // binfmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &[
                    "binfmt-error",
                    "binfmt-warn",
                    "binfmt-info",
                    "binfmt-debug",
                    "binfmt-trace",
                    "binfmt-default",
                ]
            }
        }
    }
}

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

            if de.variants.is_empty() {
                exprs.push(quote!(match *self {}));
            } else {
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
                            #(#exprs;)*
                        }
                    ))
                }

                let sym = mksym(&fs, "fmt");
                exprs.push(quote!(
                    if f.needs_tag() {
                        f.istr(&binfmt::export::istr(#sym));
                    }
                ));
                exprs.push(quote!(match self {
                    #(#arms)*
                }));
            }
        }

        Data::Struct(ds) => {
            fs = ident.to_string();
            let args = fields(&ds.fields, &mut fs, Kind::Struct);
            // FIXME expand this `write!` and conditionally omit the tag (string index)
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
                    let ty = as_native_type(&f.ty).unwrap_or_else(|| "?".to_string());
                    if let Some(ident) = f.ident.as_ref() {
                        core::write!(format, "{}: {{:{}}}", ident, ty).ok();

                        match &kind {
                            Kind::Struct => {
                                list.push(quote!(self.#ident));
                            }
                            Kind::Enum { .. } => {
                                if ty == "?" {
                                    list.push(quote!(f.fmt(#ident, false)));
                                } else {
                                    let method = format_ident!("{}", ty);
                                    list.push(quote!(f.#method(#ident)));
                                }
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
                                if ty == "?" {
                                    list.push(quote!(f.fmt(#ident, false)));
                                } else {
                                    let method = format_ident!("{}", ty);
                                    list.push(quote!(f.#method(#ident)));
                                }

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
                        **patterns = quote!({ #(#pats),* })
                    } else {
                        **patterns = quote!((#(#pats),*))
                    }
                }
            }
        }

        Fields::Unit => {}
    }

    list
}

/// Returns `true` if `ty_name` refers to a builtin Rust type that has native support from binfmt
/// and does not have to go through the `Format` trait.
///
/// This should return `true` for all types that can be used as `{:type}`.
///
/// Note: This is technically incorrect, since builtin types can be shadowed. However the efficiency
/// gains are too big to pass up, so we expect user code to not do that.
fn as_native_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => match p.path.get_ident() {
            Some(ident) => {
                let s = ident.to_string();
                match &*s {
                    "u8" | "u16" | "u32" | "usize" | "i8" | "i16" | "i32" | "isize" | "f32"
                    | "bool" => Some(s),
                    _ => None,
                }
            }
            None => None,
        },
        _ => None,
    }
}

fn is_logging_enabled(level: MLevel) -> TokenStream2 {
    let features_dev = level.necessary_features(true);
    let features_release = level.necessary_features(false);

    quote!(
        cfg!(debug_assertions) && cfg!(any(#( feature = #features_dev ),*))
            || !cfg!(debug_assertions) && cfg!(any(#( feature = #features_release ),*))
    )
}

// note that we are not using the `Level` type because we want to avoid dependencies on
// `binfmt-common` due to Cargo bugs in crate sharing
fn log(level: MLevel, ts: TokenStream) -> TokenStream {
    let log = parse_macro_input!(ts as Log);
    let ls = log.litstr.value();
    let fragments = match binfmt_parser::parse(&ls) {
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

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), log.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let sym = mksym(&ls, level.as_str());
    let logging_enabled = is_logging_enabled(level);
    quote!({
        if #logging_enabled {
            if let Some(mut _fmt_) = binfmt::export::acquire() {
                match (binfmt::export::timestamp(), #(&(#args)),*) {
                    (ts, #(#pats),*) => {
                        _fmt_.istr(&binfmt::export::istr(#sym));
                        _fmt_.leb64(ts);
                        #(#exprs;)*
                        _fmt_.finalize();
                        binfmt::export::release(_fmt_)
                    }
                }
            }
        }
    })
    .into()
}

#[proc_macro]
pub fn trace(ts: TokenStream) -> TokenStream {
    log(MLevel::Trace, ts)
}

#[proc_macro]
pub fn debug(ts: TokenStream) -> TokenStream {
    log(MLevel::Debug, ts)
}

#[proc_macro]
pub fn info(ts: TokenStream) -> TokenStream {
    log(MLevel::Info, ts)
}

#[proc_macro]
pub fn warn(ts: TokenStream) -> TokenStream {
    log(MLevel::Warn, ts)
}

#[proc_macro]
pub fn error(ts: TokenStream) -> TokenStream {
    log(MLevel::Error, ts)
}

// TODO share more code with `log`
#[proc_macro]
pub fn winfo(ts: TokenStream) -> TokenStream {
    let write = parse_macro_input!(ts as Write);
    let ls = write.litstr.value();
    let fragments = match binfmt_parser::parse(&ls) {
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

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), write.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let f = &write.fmt;
    let sym = mksym(&ls, "info");
    quote!({
        match (&mut #f, binfmt::export::timestamp(), #(&(#args)),*) {
            (_fmt_, ts, #(#pats),*) => {
                _fmt_.istr(&binfmt::export::istr(#sym));
                _fmt_.leb64(ts);
                #(#exprs;)*
                _fmt_.finalize();
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
                Some((input.parse()?, Punctuated::parse_terminated(input)?))
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

    let sym = mksym(&ls, "str");
    quote!({
        binfmt::export::istr(#sym)
    })
    .into()
}

// TODO(likely) remove
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
    quote!(match () {
        #[cfg(target_arch = "x86_64")]
        () => {
            binfmt::export::fetch_add_string_index() as u8
        }
        #[cfg(not(target_arch = "x86_64"))]
        () => {
            #[link_section = #section]
            #[export_name = #sym]
            static S: u8 = 0;
            &S as *const u8 as u8
        }
    })
    .into()
}

#[proc_macro]
pub fn write(ts: TokenStream) -> TokenStream {
    let write = parse_macro_input!(ts as Write);
    let ls = write.litstr.value();
    let fragments = match binfmt_parser::parse(&ls) {
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

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), write.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error().into(),
    };

    let fmt = &write.fmt;
    let sym = mksym(&ls, "fmt");
    quote!(match (#fmt, #(&(#args)),*) {
        (ref mut _fmt_, #(#pats),*) => {
            // HACK conditional should not be here; see FIXME in `format`
            if _fmt_.needs_tag() {
                _fmt_.istr(&binfmt::export::istr(#sym));
            }
            #(#exprs;)*
            _fmt_.finalize();
        }
    })
    .into()
}

fn mksym(string: &str, section: &str) -> TokenStream2 {
    let id = format!("{:?}", Span::call_site());
    let section = format!(".binfmt.{}.{}", section, string);
    let sym = format!("{}@{}", string, id);
    quote!(match () {
        #[cfg(target_arch = "x86_64")]
        () => {
            binfmt::export::fetch_add_string_index()
        }
        #[cfg(not(target_arch = "x86_64"))]
        () => {
            #[link_section = #section]
            #[export_name = #sym]
            static S: u8 = 0;
            &S as *const u8 as usize
        }
    })
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
                Some((input.parse()?, Punctuated::parse_terminated(input)?))
            },
        })
    }
}

struct Codegen {
    pats: Vec<Ident2>,
    exprs: Vec<TokenStream2>,
}

impl Codegen {
    fn new(fragments: &Vec<Fragment<'_>>, num_args: usize, span: Span2) -> parse::Result<Self> {
        let parsed_params = fragments
            .iter()
            .filter_map(|frag| match frag {
                Fragment::Parameter(param) => Some(param.clone()),
                Fragment::Literal(_) => None,
            })
            .collect::<Vec<_>>();

        let actual_param_count = parsed_params
            .iter()
            .map(|param| param.index + 1)
            .max()
            .unwrap_or(0);

        let mut exprs = vec![];
        let mut pats = vec![];

        for i in 0..actual_param_count {
            let arg = format_ident!("arg{}", i);
            let param = parsed_params.iter().find(|param| param.index == i).unwrap();
            match param.ty {
                binfmt_parser::Type::Format => {
                    exprs.push(quote!(_fmt_.fmt(#arg, false)));
                }
                binfmt_parser::Type::FormatSlice => {
                    exprs.push(quote!(_fmt_.fmt_slice(#arg)));
                }
                binfmt_parser::Type::I16 => {
                    exprs.push(quote!(_fmt_.i16(#arg)));
                }
                binfmt_parser::Type::I32 => {
                    exprs.push(quote!(_fmt_.i32(#arg)));
                }
                binfmt_parser::Type::I8 => {
                    exprs.push(quote!(_fmt_.i8(#arg)));
                }
                binfmt_parser::Type::Isize => {
                    exprs.push(quote!(_fmt_.isize(#arg)));
                }
                binfmt_parser::Type::Str => {
                    exprs.push(quote!(_fmt_.str(#arg)));
                }
                binfmt_parser::Type::IStr => {
                    exprs.push(quote!(_fmt_.istr(#arg)));
                }
                binfmt_parser::Type::U16 => {
                    exprs.push(quote!(_fmt_.u16(#arg)));
                }
                binfmt_parser::Type::U24 => {
                    exprs.push(quote!(_fmt_.u24(#arg)));
                }
                binfmt_parser::Type::U32 => {
                    exprs.push(quote!(_fmt_.u32(#arg)));
                }
                binfmt_parser::Type::U8 => {
                    exprs.push(quote!(_fmt_.u8(#arg)));
                }
                binfmt_parser::Type::Usize => {
                    exprs.push(quote!(_fmt_.usize(#arg)));
                }
                binfmt_parser::Type::BitField(_) => {
                    todo!();
                }
                binfmt_parser::Type::Bool => {
                    exprs.push(quote!(_fmt_.bool(#arg)));
                }
                binfmt_parser::Type::Slice => {
                    exprs.push(quote!(_fmt_.slice(#arg)));
                }
                binfmt_parser::Type::Array(len) => {
                    // We cast to the expected array type (which should be a no-op cast) to provoke
                    // a type mismatch error on mismatched lengths:
                    // ```
                    // error[E0308]: mismatched types
                    //   --> src/bin/log.rs:20:5
                    //    |
                    // 20 |     binfmt::info!("🍕 array {:[u8; 3]}", [3, 14]);
                    //    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
                    //    |     |
                    //    |     expected an array with a fixed size of 3 elements, found one with 2 elements
                    //    |     expected due to this
                    // ```
                    exprs.push(quote!(_fmt_.array({
                        let tmp: &[u8; #len] = #arg;
                        tmp
                    })));
                }
                binfmt_parser::Type::F32 => {
                    exprs.push(quote!(_fmt_.f32(#arg)));
                }
            }
            pats.push(arg);
        }

        if num_args < actual_param_count {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but only {} were provided",
                    actual_param_count, num_args
                ),
            ));
        }

        if num_args > actual_param_count {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but {} were provided",
                    actual_param_count, num_args
                ),
            ));
        }

        Ok(Codegen { pats, exprs })
    }
}
