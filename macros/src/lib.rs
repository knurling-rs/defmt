use core::fmt::Write as _;
use proc_macro::{Span, TokenStream};

use defmt_parser::Fragment;
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
        unsafe fn _defmt_acquire() -> Option<defmt::Formatter> {
            <#ident as defmt::Logger>::acquire().map(|nn| defmt::Formatter::from_raw(nn))
        }

        #[no_mangle]
        unsafe fn _defmt_release(f: defmt::Formatter)  {
            <#ident as defmt::Logger>::release(f.into_raw())
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
        #[export_name = "_defmt_timestamp"]
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
                    &["defmt-trace", "defmt-default"]
                } else {
                    &["defmt-trace"]
                }
            }
            MLevel::Debug => {
                if debug_assertions {
                    // dev profile
                    &["defmt-debug", "defmt-trace", "defmt-default"]
                } else {
                    &["defmt-debug", "defmt-trace"]
                }
            }
            MLevel::Info => {
                // defmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &["defmt-info", "defmt-debug", "defmt-trace", "defmt-default"]
            }
            MLevel::Warn => {
                // defmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &[
                    "defmt-warn",
                    "defmt-info",
                    "defmt-debug",
                    "defmt-trace",
                    "defmt-default",
                ]
            }
            MLevel::Error => {
                // defmt-default is enabled for dev & release profile so debug_assertions
                // does not matter
                &[
                    "defmt-error",
                    "defmt-warn",
                    "defmt-info",
                    "defmt-debug",
                    "defmt-trace",
                    "defmt-default",
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
    let mut field_types = vec![];
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
                        &mut field_types,
                        Kind::Enum {
                            patterns: &mut pats,
                        },
                    );

                    arms.push(quote!(
                        #ident::#vident #pats => {
                            f.u8(&#i);
                            f.with_tag(|f| {
                                #(#exprs;)*
                            });
                        }
                    ))
                }

                let sym = mksym(&fs, "fmt");
                exprs.push(quote!(
                    if f.needs_tag() {
                        f.istr(&defmt::export::istr(#sym));
                    }
                ));
                exprs.push(quote!(match self {
                    #(#arms)*
                }));
            }
        }

        Data::Struct(ds) => {
            fs = ident.to_string();
            let args = fields(&ds.fields, &mut fs, &mut field_types, Kind::Struct);
            // FIXME expand this `write!` and conditionally omit the tag (string index)
            exprs.push(quote!(defmt::export::write!(f, #fs #(,#args)*);))
        }

        Data::Union(..) => {
            return parse::Error::new(span, "`#[derive(Format)]` does not support unions")
                .to_compile_error()
                .into();
        }
    }

    let params = input.generics.params;
    let predicates = if params.is_empty() {
        vec![]
    } else {
        // `Format` bounds for non-native field types
        let mut preds = field_types
            .into_iter()
            .map(|ty| quote!(#ty: defmt::Format))
            .collect::<Vec<_>>();
        // extend with the where clause from the struct/enum declaration
        if let Some(where_clause) = input.generics.where_clause {
            preds.extend(
                where_clause
                    .predicates
                    .into_iter()
                    .map(|pred| quote!(#pred)),
            )
        }
        preds
    };
    quote!(
        impl<#params> defmt::Format for #ident<#params>
        where #(#predicates),*
        {
            fn format(&self, f: &mut defmt::Formatter) {
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

fn fields(
    fields: &Fields,
    format: &mut String,
    // collect all *non-native* types that appear as fields
    field_types: &mut Vec<Type>,
    mut kind: Kind,
) -> Vec<TokenStream2> {
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
                    let ty = as_native_type(&f.ty).unwrap_or_else(|| {
                        field_types.push(f.ty.clone());
                        "?".to_string()
                    });
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

/// Returns `true` if `ty_name` refers to a builtin Rust type that has native support from defmt
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
// `defmt-common` due to Cargo bugs in crate sharing
fn log(level: MLevel, ts: TokenStream) -> TokenStream {
    let log = parse_macro_input!(ts as Log);
    let ls = log.litstr.value();
    let fragments = match defmt_parser::parse(&ls) {
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
            if let Some(mut _fmt_) = defmt::export::acquire() {
                match (defmt::export::timestamp(), #(&(#args)),*) {
                    (ts, #(#pats),*) => {
                        _fmt_.istr(&defmt::export::istr(#sym));
                        _fmt_.leb64(ts);
                        #(#exprs;)*
                        _fmt_.finalize();
                        defmt::export::release(_fmt_)
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
    let fragments = match defmt_parser::parse(&ls) {
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
        match (&mut #f, defmt::export::timestamp(), #(&(#args)),*) {
            (_fmt_, ts, #(#pats),*) => {
                _fmt_.istr(&defmt::export::istr(#sym));
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
        defmt::export::istr(#sym)
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
    let section = format!(".defmt.prim.{}", ls);
    let sym = ls;
    quote!(match () {
        #[cfg(target_arch = "x86_64")]
        () => {
            defmt::export::fetch_add_string_index() as u8
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
    let fragments = match defmt_parser::parse(&ls) {
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
                _fmt_.istr(&defmt::export::istr(#sym));
            }
            #(#exprs;)*
            _fmt_.finalize();
        }
    })
    .into()
}

fn mksym(string: &str, section: &str) -> TokenStream2 {
    let id = format!("{:?}", Span::call_site());
    let section = format!(".defmt.{}.{}", section, string);
    let sym = format!("{}@{}", string, id);
    quote!(match () {
        #[cfg(target_arch = "x86_64")]
        () => {
            defmt::export::fetch_add_string_index()
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

        let actual_argument_count = parsed_params
            .iter()
            .map(|param| param.index + 1)
            .max()
            .unwrap_or(0);

        let mut exprs = vec![];
        let mut pats = vec![];

        for i in 0..actual_argument_count {
            let arg = format_ident!("arg{}", i);
            // find first use of this argument and return its type
            let param = parsed_params.iter().find(|param| param.index == i).unwrap();
            match param.ty {
                defmt_parser::Type::Format => {
                    exprs.push(quote!(_fmt_.fmt(#arg, false)));
                }
                defmt_parser::Type::FormatSlice => {
                    exprs.push(quote!(_fmt_.fmt_slice(#arg)));
                }
                defmt_parser::Type::I16 => {
                    exprs.push(quote!(_fmt_.i16(#arg)));
                }
                defmt_parser::Type::I32 => {
                    exprs.push(quote!(_fmt_.i32(#arg)));
                }
                defmt_parser::Type::I8 => {
                    exprs.push(quote!(_fmt_.i8(#arg)));
                }
                defmt_parser::Type::Isize => {
                    exprs.push(quote!(_fmt_.isize(#arg)));
                }
                defmt_parser::Type::Str => {
                    exprs.push(quote!(_fmt_.str(#arg)));
                }
                defmt_parser::Type::IStr => {
                    exprs.push(quote!(_fmt_.istr(#arg)));
                }
                defmt_parser::Type::U16 => {
                    exprs.push(quote!(_fmt_.u16(#arg)));
                }
                defmt_parser::Type::U24 => {
                    exprs.push(quote!(_fmt_.u24(#arg)));
                }
                defmt_parser::Type::U32 => {
                    exprs.push(quote!(_fmt_.u32(#arg)));
                }
                defmt_parser::Type::U8 => {
                    exprs.push(quote!(_fmt_.u8(#arg)));
                }
                defmt_parser::Type::Usize => {
                    exprs.push(quote!(_fmt_.usize(#arg)));
                }
                defmt_parser::Type::BitField(_) => {
                    // TODO reused in decoder::parse_args(), can we share this somehow without Cargo bug troubles?
                    let all_bitfields = parsed_params.iter().filter(|param| param.index == i);
                    let largest_bit_index = all_bitfields
                        .map(|param| match &param.ty {
                            defmt_parser::Type::BitField(range) => range.end,
                            _ => unreachable!(),
                        })
                        .max()
                        .unwrap();

                    match largest_bit_index {
                        0..=8 => {
                            exprs.push(quote!(_fmt_.u8(&defmt::export::truncate(*#arg))));
                        }
                        9..=16 => {
                            exprs.push(quote!(_fmt_.u16(&defmt::export::truncate(*#arg))));
                        }
                        17..=24 => {
                            exprs.push(quote!(_fmt_.u24(&defmt::export::truncate(*#arg))));
                        }
                        25..=32 => {
                            exprs.push(quote!(_fmt_.u32(&defmt::export::truncate(*#arg))));
                        }
                        _ => unreachable!(),
                    }
                }
                defmt_parser::Type::Bool => {
                    exprs.push(quote!(_fmt_.bool(#arg)));
                }
                defmt_parser::Type::Slice => {
                    exprs.push(quote!(_fmt_.slice(#arg)));
                }
                defmt_parser::Type::Array(len) => {
                    // We cast to the expected array type (which should be a no-op cast) to provoke
                    // a type mismatch error on mismatched lengths:
                    // ```
                    // error[E0308]: mismatched types
                    //   --> src/bin/log.rs:20:5
                    //    |
                    // 20 |     defmt::info!("🍕 array {:[u8; 3]}", [3, 14]);
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
                defmt_parser::Type::F32 => {
                    exprs.push(quote!(_fmt_.f32(#arg)));
                }
            }
            pats.push(arg);
        }

        if num_args < actual_argument_count {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but only {} were provided",
                    actual_argument_count, num_args
                ),
            ));
        }

        if num_args > actual_argument_count {
            return Err(parse::Error::new(
                span,
                format!(
                    "format string requires {} arguments but {} were provided",
                    actual_argument_count, num_args
                ),
            ));
        }

        Ok(Codegen { pats, exprs })
    }
}
