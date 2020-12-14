//! INTERNAL; DO NOT USE. Please use the `defmt` crate to access the functionality implemented here
mod symbol;

use core::fmt::Write as _;
use proc_macro::TokenStream;

use defmt_parser::{Fragment, Level};
use proc_macro2::{Ident as Ident2, Span as Span2, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned as _,
    Data, DeriveInput, Expr, ExprPath, Fields, FieldsNamed, FieldsUnnamed, GenericParam, ItemFn,
    ItemStruct, LitStr, Path, PathArguments, PathSegment, ReturnType, Token, Type, WhereClause,
    WherePredicate,
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
pub fn panic_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(
            Span2::call_site(),
            "`#[defmt::panic_handler]` attribute takes no arguments",
        )
        .to_compile_error()
        .into();
    }
    let f = parse_macro_input!(input as ItemFn);

    let rety_is_ok = match &f.sig.output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match &**ty {
            Type::Never(_) => true,
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
        return parse::Error::new(ident.span(), "function must have signature `fn() -> !`")
            .to_compile_error()
            .into();
    }

    let block = &f.block;
    quote!(
        #[export_name = "_defmt_panic"]
        fn #ident() -> ! {
            #block
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

// returns a list of features of which one has to be enabled for `level` to be active
fn necessary_features_for_level(level: Level, debug_assertions: bool) -> &'static [&'static str] {
    match level {
        Level::Trace => {
            if debug_assertions {
                // dev profile
                &["defmt-trace", "defmt-default"]
            } else {
                &["defmt-trace"]
            }
        }
        Level::Debug => {
            if debug_assertions {
                // dev profile
                &["defmt-debug", "defmt-trace", "defmt-default"]
            } else {
                &["defmt-debug", "defmt-trace"]
            }
        }
        Level::Info => {
            // defmt-default is enabled for dev & release profile so debug_assertions
            // does not matter
            &["defmt-info", "defmt-debug", "defmt-trace", "defmt-default"]
        }
        Level::Warn => {
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
        Level::Error => {
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

// `#[derive(Format)]`
#[proc_macro_derive(Format)]
pub fn format(ts: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(ts as DeriveInput);
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
                // For zero-variant enums, this is unreachable code.
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

                    let mut pats = vec![];
                    let exprs = fields(&var.fields, &mut fs, &mut field_types, &mut pats);
                    let pats = quote!( { #(#pats),* } );

                    let encode_discriminant = if de.variants.len() == 1 {
                        // For single-variant enums, there is no need to encode the discriminant.
                        quote!()
                    } else {
                        quote!(
                            f.u8(&#i);
                        )
                    };

                    arms.push(quote!(
                        #ident::#vident #pats => {
                            #encode_discriminant

                            // When descending into an enum variant, force all discriminants to be
                            // encoded. This is required when encoding arrays like `[None, Some(x)]`
                            // with `{:?}`, since the format string of `x` won't appear for the
                            // first element.
                            f.with_tag(|f| {
                                #(#exprs;)*
                            });
                        }
                    ))
                }

                let sym = mksym(&fs, "fmt", false);
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
            let mut pats = vec![];
            let args = fields(&ds.fields, &mut fs, &mut field_types, &mut pats);

            let sym = mksym(&fs, "fmt", false);
            exprs.push(quote!(
                if f.needs_tag() {
                    f.istr(&defmt::export::istr(#sym));
                }
            ));
            exprs.push(quote!(match self {
                Self { #(#pats),* } => {
                    #(#args;)*
                }
            }));
        }

        Data::Union(..) => {
            return parse::Error::new(span, "`#[derive(Format)]` does not support unions")
                .to_compile_error()
                .into();
        }
    }

    let where_clause = input.generics.make_where_clause();
    let mut where_clause: WhereClause = where_clause.clone();
    let (impl_generics, type_generics, _) = input.generics.split_for_impl();

    // Extend where-clause with `Format` bounds for type parameters.
    for param in &input.generics.params {
        if let GenericParam::Type(ty) = param {
            let ident = &ty.ident;
            where_clause
                .predicates
                .push(syn::parse::<WherePredicate>(quote!(#ident: defmt::Format).into()).unwrap());
        }
    }

    quote!(
        impl #impl_generics defmt::Format for #ident #type_generics #where_clause {
            fn format(&self, f: &mut defmt::Formatter) {
                #(#exprs)*
            }
        }
    )
    .into()
}

fn fields(
    fields: &Fields,
    format: &mut String,
    // collect all *non-native* types that appear as fields
    field_types: &mut Vec<Type>,
    pats: &mut Vec<TokenStream2>,
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

                        if ty == "?" {
                            list.push(quote!(f.fmt(#ident, false)));
                        } else {
                            let method = format_ident!("{}", ty);
                            list.push(quote!(f.#method(#ident)));
                        }
                        pats.push(quote!( #ident ));
                    } else {
                        // Unnamed (tuple) field.

                        core::write!(format, "{{:{}}}", ty).ok();

                        let ident = format_ident!("arg{}", i);
                        if ty == "?" {
                            list.push(quote!(f.fmt(#ident, false)));
                        } else {
                            let method = format_ident!("{}", ty);
                            list.push(quote!(f.#method(#ident)));
                        }

                        let i = syn::Index::from(i);
                        pats.push(quote!( #i: #ident ));
                    }
                }
                if named {
                    format.push_str(" }}");
                } else {
                    format.push_str(")");
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
        Type::Reference(tref) => as_native_type(&*tref.elem),
        _ => None,
    }
}

fn is_logging_enabled(level: Level) -> TokenStream2 {
    let features_dev = necessary_features_for_level(level, true);
    let features_release = necessary_features_for_level(level, false);

    quote!(
        cfg!(debug_assertions) && cfg!(any(#( feature = #features_dev ),*))
            || !cfg!(debug_assertions) && cfg!(any(#( feature = #features_release ),*))
    )
}

// note that we are not using a `Level` type shared with `decoder` due to Cargo bugs in crate sharing
// TODO -> move Level to parser?
fn log_ts(level: Level, ts: TokenStream) -> TokenStream {
    log(level, parse_macro_input!(ts as FormatArgs)).into()
}

fn log(level: Level, log: FormatArgs) -> TokenStream2 {
    let ls = log.litstr.value();
    let fragments = match defmt_parser::parse(&ls) {
        Ok(args) => args,
        Err(e) => return parse::Error::new(log.litstr.span(), e).to_compile_error(),
    };

    let args = log
        .rest
        .map(|(_, exprs)| exprs.into_iter().collect())
        .unwrap_or_else(Vec::new);

    let (pats, exprs) = match Codegen::new(&fragments, args.len(), log.litstr.span()) {
        Ok(cg) => (cg.pats, cg.exprs),
        Err(e) => return e.to_compile_error(),
    };

    let sym = mksym(&ls, level.as_str(), true);
    let logging_enabled = is_logging_enabled(level);
    quote!({
        if #logging_enabled {
            match (#(&(#args)),*) {
                (#(#pats),*) => {
                    if let Some(mut _fmt_) = defmt::export::acquire() {
                        _fmt_.header(&defmt::export::istr(#sym));
                        #(#exprs;)*
                        _fmt_.finalize();
                        defmt::export::release(_fmt_)
                    }
                }
            }
        }
    })
}

#[proc_macro]
pub fn trace(ts: TokenStream) -> TokenStream {
    log_ts(Level::Trace, ts)
}

#[proc_macro]
pub fn debug(ts: TokenStream) -> TokenStream {
    log_ts(Level::Debug, ts)
}

#[proc_macro]
pub fn info(ts: TokenStream) -> TokenStream {
    log_ts(Level::Info, ts)
}

#[proc_macro]
pub fn warn(ts: TokenStream) -> TokenStream {
    log_ts(Level::Warn, ts)
}

#[proc_macro]
pub fn error(ts: TokenStream) -> TokenStream {
    log_ts(Level::Error, ts)
}

fn panic(
    ts: TokenStream,
    zero_args_string: &str,
    string_transform: impl FnOnce(&str) -> String,
) -> TokenStream {
    let log_stmt = if ts.is_empty() {
        // panic!() -> error!("panicked at 'explicit panic'")
        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(zero_args_string, Span2::call_site()),
                rest: None,
            },
        )
    } else {
        // panic!("a", b, c) -> error!("panicked at 'a'", b, c)
        let args = parse_macro_input!(ts as FormatArgs);
        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(&string_transform(&args.litstr.value()), Span2::call_site()),
                rest: args.rest,
            },
        )
    };

    quote!(
        {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}

// not naming this `panic` to avoid shadowing `core::panic` in this scope
#[proc_macro]
pub fn panic_(ts: TokenStream) -> TokenStream {
    panic(ts, "panicked at 'explicit panic'", |format_string| {
        format!("panicked at '{}'", format_string)
    })
}

// not naming this `todo` to avoid shadowing `core::todo` in this scope
#[proc_macro]
pub fn todo_(ts: TokenStream) -> TokenStream {
    panic(ts, "panicked at 'not yet implemented'", |format_string| {
        format!("panicked at 'not yet implemented: {}'", format_string)
    })
}

// not naming this `unreachable` to avoid shadowing `core::unreachable` in this scope
#[proc_macro]
pub fn unreachable_(ts: TokenStream) -> TokenStream {
    panic(
        ts,
        "panicked at 'internal error: entered unreachable code'",
        |format_string| {
            format!(
                "panicked at 'internal error: entered unreachable code: {}'",
                format_string
            )
        },
    )
}

// not naming this `assert` to avoid shadowing `core::assert` in this scope
#[proc_macro]
pub fn assert_(ts: TokenStream) -> TokenStream {
    let assert = parse_macro_input!(ts as Assert);

    let condition = assert.condition;
    let log_stmt = if let Some(args) = assert.args {
        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!("panicked at '{}'", args.litstr.value()),
                    Span2::call_site(),
                ),
                rest: args.rest,
            },
        )
    } else {
        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!(
                        "panicked at 'assertion failed: {}'",
                        escape_expr(&condition)
                    ),
                    Span2::call_site(),
                ),
                rest: None,
            },
        )
    };

    quote!(
        if !(#condition) {
            #log_stmt;
            defmt::export::panic()
        }
    )
    .into()
}

#[derive(PartialEq)]
enum BinOp {
    Eq,
    Ne,
}

// not naming this `assert_eq` to avoid shadowing `core::assert_eq` in this scope
#[proc_macro]
pub fn assert_eq_(ts: TokenStream) -> TokenStream {
    assert_binop(ts, BinOp::Eq)
}

// not naming this `assert_ne` to avoid shadowing `core::assert_ne` in this scope
#[proc_macro]
pub fn assert_ne_(ts: TokenStream) -> TokenStream {
    assert_binop(ts, BinOp::Ne)
}

// not naming this `assert_eq` to avoid shadowing `core::assert_eq` in this scope
fn assert_binop(ts: TokenStream, binop: BinOp) -> TokenStream {
    let assert = parse_macro_input!(ts as AssertEq);

    let left = assert.left;
    let right = assert.right;

    let mut log_args = Punctuated::new();

    let extra_string = if let Some(args) = assert.args {
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
        log_args.push(ident_expr(*val));
    }

    let log_stmt = match binop {
        BinOp::Eq => log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!(
                        "panicked at 'assertion failed: `(left == right)`{}'
 left: `{{:?}}`
right: `{{:?}}`",
                        extra_string
                    ),
                    Span2::call_site(),
                ),
                rest: Some((syn::token::Comma::default(), log_args)),
            },
        ),
        BinOp::Ne => log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!(
                        "panicked at 'assertion failed: `(left != right)`{}'
left/right: `{{:?}}`",
                        extra_string
                    ),
                    Span2::call_site(),
                ),
                rest: Some((syn::token::Comma::default(), log_args)),
            },
        ),
    };

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

// NOTE these `debug_*` macros can be written using `macro_rules!` (that'd be simpler) but that
// results in an incorrect source code location being reported: the location of the `macro_rules!`
// statement is reported. Using a proc-macro results in the call site being reported, which is what
// we want
#[proc_macro]
pub fn debug_assert_(ts: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_(ts));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro]
pub fn debug_assert_eq_(ts: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_eq_(ts));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro]
pub fn debug_assert_ne_(ts: TokenStream) -> TokenStream {
    let assert = TokenStream2::from(assert_ne_(ts));
    quote!(if cfg!(debug_assertions) {
        #assert
    })
    .into()
}

#[proc_macro]
pub fn unwrap(ts: TokenStream) -> TokenStream {
    let assert = parse_macro_input!(ts as Assert);

    let condition = assert.condition;
    let log_stmt = if let Some(args) = assert.args {
        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!("panicked at '{}'", args.litstr.value()),
                    Span2::call_site(),
                ),
                rest: args.rest,
            },
        )
    } else {
        let mut log_args = Punctuated::new();
        log_args.push(ident_expr("_unwrap_err"));

        log(
            Level::Error,
            FormatArgs {
                litstr: LitStr::new(
                    &format!(
                        "panicked at 'unwrap failed: {}'
error: `{{:?}}`",
                        escape_expr(&condition)
                    ),
                    Span2::call_site(),
                ),
                rest: Some((syn::token::Comma::default(), log_args)),
            },
        )
    };

    quote!(
        match defmt::export::into_result(#condition) {
            ::core::result::Result::Ok(res) => res,
            ::core::result::Result::Err(_unwrap_err) => {
                #log_stmt;
                defmt::export::panic()
            }
        }
    )
    .into()
}

fn ident_expr(name: &str) -> Expr {
    let mut segments = Punctuated::new();
    segments.push(PathSegment {
        ident: Ident2::new(name, Span2::call_site()),
        arguments: PathArguments::None,
    });

    Expr::Path(ExprPath {
        attrs: vec![],
        qself: None,
        path: Path {
            leading_colon: None,
            segments,
        },
    })
}

fn escape_expr(expr: &Expr) -> String {
    let q = quote!(#expr);
    q.to_string().replace("{", "{{").replace("}", "}}")
}

struct Assert {
    condition: Expr,
    args: Option<FormatArgs>,
}

impl Parse for Assert {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let condition = input.parse()?;
        if input.is_empty() {
            // assert!(a)
            return Ok(Assert {
                condition,
                args: None,
            });
        }

        let _comma: Token![,] = input.parse()?;

        if input.is_empty() {
            // assert!(a,)
            Ok(Assert {
                condition,
                args: None,
            })
        } else {
            // assert!(a, "b", c)
            Ok(Assert {
                condition,
                args: Some(input.parse()?),
            })
        }
    }
}

struct AssertEq {
    left: Expr,
    right: Expr,
    args: Option<FormatArgs>,
}

impl Parse for AssertEq {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let left = input.parse()?;
        let _comma: Token![,] = input.parse()?;
        let right = input.parse()?;

        if input.is_empty() {
            // assert_eq!(a, b)
            return Ok(AssertEq {
                left,
                right,
                args: None,
            });
        }

        let _comma: Token![,] = input.parse()?;

        if input.is_empty() {
            // assert_eq!(a, b,)
            Ok(AssertEq {
                left,
                right,
                args: None,
            })
        } else {
            // assert_eq!(a, b, "c", d)
            Ok(AssertEq {
                left,
                right,
                args: Some(input.parse()?),
            })
        }
    }
}

struct FormatArgs {
    litstr: LitStr,
    rest: Option<(Token![,], Punctuated<Expr, Token![,]>)>,
}

impl Parse for FormatArgs {
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

    let sym = mksym(&ls, "str", false);
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

    let sym = symbol::Symbol::new("prim", &ls).mangle();
    let section = format!(".defmt.prim.{}", sym);

    if cfg!(feature = "unstable-test") {
        quote!({ defmt::export::fetch_add_string_index() as u8 })
    } else {
        quote!({
            #[link_section = #section]
            #[export_name = #sym]
            static S: u8 = 0;
            &S as *const u8 as u8
        })
    }
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
    // FIXME: Introduce a new `"write"` tag and decode it in a loop (breaking change).
    let sym = mksym(&ls, "fmt", false);
    quote!(match (&mut *#fmt, #(&(#args)),*) {
        (_fmt_, #(#pats),*) => {
            _fmt_.write_macro_start();
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

fn mksym(string: &str, tag: &str, is_log_statement: bool) -> TokenStream2 {
    let sym = symbol::Symbol::new(tag, string).mangle();
    let section = format!(".defmt.{}", sym);

    // NOTE we rely on this variable name when extracting file location information from the DWARF
    // without it we have no other mean to differentiate static variables produced by `info!` vs
    // produced by `intern!` (or `internp`)
    let varname = if is_log_statement {
        format_ident!("DEFMT_LOG_STATEMENT")
    } else {
        format_ident!("S")
    };
    if cfg!(feature = "unstable-test") {
        quote!({ defmt::export::fetch_add_string_index() })
    } else {
        quote!({
            #[link_section = #section]
            #[export_name = #sym]
            static #varname: u8 = 0;
            &#varname as *const u8 as usize
        })
    }
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
                defmt_parser::Type::I64 => {
                    exprs.push(quote!(_fmt_.i64(#arg)));
                }
                defmt_parser::Type::I128 => {
                    exprs.push(quote!(_fmt_.i128(#arg)));
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
                defmt_parser::Type::U64 => {
                    exprs.push(quote!(_fmt_.u64(#arg)));
                }
                defmt_parser::Type::U128 => {
                    exprs.push(quote!(_fmt_.u128(#arg)));
                }
                defmt_parser::Type::U8 => {
                    exprs.push(quote!(_fmt_.u8(#arg)));
                }
                defmt_parser::Type::Usize => {
                    exprs.push(quote!(_fmt_.usize(#arg)));
                }
                defmt_parser::Type::BitField(_) => {
                    let all_bitfields = parsed_params.iter().filter(|param| param.index == i);
                    let (smallest_bit_index, largest_bit_index) =
                        defmt_parser::get_max_bitfield_range(all_bitfields).unwrap();

                    // indices of the lowest and the highest octet which contains bitfield-relevant data
                    let lowest_byte = smallest_bit_index / 8;
                    let highest_byte = (largest_bit_index - 1) / 8;
                    let truncated_sz = highest_byte - lowest_byte + 1; // in bytes

                    match truncated_sz {
                        1 => {
                            // shift away unneeded lower octet
                            // TODO: create helper for shifting because readability
                            exprs.push(quote!(_fmt_.u8(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8)))));
                        }
                        2 => {
                            exprs.push(quote!(_fmt_.u16(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8)))));
                        }
                        3 => {
                            exprs.push(quote!(_fmt_.u24(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8)))));
                        }
                        4 => {
                            exprs.push(quote!(_fmt_.u32(&defmt::export::truncate((*#arg) >> (#lowest_byte * 8)))));
                        }
                        _ => unreachable!(),
                    }
                }
                defmt_parser::Type::Bool => {
                    exprs.push(quote!(_fmt_.bool(#arg)));
                }
                defmt_parser::Type::U8Slice => {
                    exprs.push(quote!(_fmt_.slice(#arg)));
                }
                defmt_parser::Type::U8Array(len) => {
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
                    exprs.push(quote!(_fmt_.u8_array({
                        let tmp: &[u8; #len] = #arg;
                        tmp
                    })));
                }
                defmt_parser::Type::FormatArray(len) => {
                    exprs.push(quote!(_fmt_.fmt_array({
                        let tmp: &[_; #len] = #arg;
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
