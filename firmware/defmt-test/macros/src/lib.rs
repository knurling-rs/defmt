extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse, spanned::Spanned, Block, FnArg, Ident, Item, ItemMod, Path, ReturnType, Type};

#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    match tests_impl(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn tests_impl(args: TokenStream, input: TokenStream) -> parse::Result<TokenStream> {
    if !args.is_empty() {
        return Err(parse::Error::new(
            Span::call_site(),
            "`#[test]` attribute takes no arguments",
        ));
    }

    let module: ItemMod = syn::parse(input)?;

    let items = if let Some(content) = module.content {
        content.1
    } else {
        return Err(parse::Error::new(
            module.span(),
            "module must be inline (e.g. `mod foo {}`)",
        ));
    };

    let mut init = None;
    let mut tests = vec![];
    let mut imports = vec![];
    for item in items {
        let item_span = item.span();
        match item {
            Item::Fn(f) => {
                // TODO span could be better here
                let attr = if let Some(attr) = f
                    .attrs
                    .iter()
                    .filter_map(|attr| {
                        if path_is_ident(&attr.path, "init") {
                            Some(Attr::Init)
                        } else if path_is_ident(&attr.path, "test") {
                            Some(Attr::Test)
                        } else {
                            None
                        }
                    })
                    .next()
                {
                    attr
                } else {
                    return Err(parse::Error::new(
                        item_span,
                        "only attributes `#[test]` and `#[init]` are accepted",
                    ));
                };

                match attr {
                    Attr::Init => {
                        if init.is_some() {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "only a single `#[init]` function can be defined",
                            ));
                        }

                        if check_fn_sig(&f.sig).is_err() || !f.sig.inputs.is_empty() {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "`#[init]` function must have signature `fn() [-> Type]` (the return type is optional)",
                            ));
                        }

                        let state = match f.sig.output {
                            ReturnType::Default => None,
                            ReturnType::Type(.., ty) => Some(ty),
                        };

                        init = Some(Init {
                            block: f.block,
                            ident: f.sig.ident,
                            state,
                        });
                    }

                    Attr::Test => {
                        if check_fn_sig(&f.sig).is_err()
                            || f.sig.output != ReturnType::Default
                            || f.sig.inputs.len() > 1
                        {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "`#[test]` function must have signature `fn([&mut Type])` (parameter is optional)",
                            ));
                        }

                        let input = if f.sig.inputs.len() == 1 {
                            let arg = &f.sig.inputs[0];

                            // NOTE we cannot check the argument type matches `init.state` at this
                            // point
                            if let Some(ty) = get_mutable_reference_type(arg).cloned() {
                                Some(Input {
                                    arg: arg.clone(),
                                    ty,
                                })
                            } else {
                                // was not `&mut T`
                                return Err(parse::Error::new(
                                    arg.span(),
                                    "parameter must be a mutable reference (`&mut $Type`)",
                                ));
                            }
                        } else {
                            None
                        };

                        tests.push(Test {
                            block: f.block,
                            ident: f.sig.ident,
                            input,
                        })
                    }
                }
            }

            Item::Use(u) => {
                imports.push(u);
            }

            _ => {
                return Err(parse::Error::new(
                    item.span(),
                    "only `#[test]` functions and imports (`use`) are allowed in this scope",
                ));
            }
        }
    }

    let krate = format_ident!("defmt_test");
    let ident = module.ident;
    let mut state_ty = None;
    let (init_fn, init_expr) = if let Some(init) = init {
        let init_ident = init.ident;
        let init_block = init.block;
        state_ty = init.state;

        (
            Some(quote!(fn #init_ident() -> #state_ty #init_block)),
            Some(quote!(#[allow(dead_code)] let mut state = #init_ident();)),
        )
    } else {
        (None, None)
    };

    let mut unit_test_calls = vec![];
    for test in &tests {
        let ident = &test.ident;
        let span = ident.span();
        let call = if let Some(input) = test.input.as_ref() {
            if let Some(state) = &state_ty {
                if input.ty != **state {
                    return Err(parse::Error::new(
                        input.ty.span(),
                        "this type must match `#[init]`s return type",
                    ));
                }
            } else {
                return Err(parse::Error::new(
                    span,
                    "no state was initialized by `#[init]`; signature must be `fn()`",
                ));
            }

            quote!(#ident(&mut state);)
        } else {
            quote!(#ident();)
        };
        unit_test_calls.push(call);
    }
    let unit_test_names = tests.iter().map(|test| &test.ident);
    let unit_test_inputs = tests
        .iter()
        .map(|test| test.input.as_ref().map(|input| &input.arg));
    let unit_test_blocks = tests.iter().map(|test| &test.block);
    let unit_test_running = tests
        .iter()
        .map(|test| format!("running {} ..", test.ident))
        .collect::<Vec<_>>();
    let unit_test_done = tests
        .iter()
        .map(|test| format!(".. {} ok", test.ident))
        .collect::<Vec<_>>();
    Ok(quote!(mod #ident {
        #(#imports)*
        // TODO use `cortex-m-rt::entry` here to get the `static mut` transform
        #[export_name = "main"]
        unsafe extern "C" fn __defmt_test_entry() -> ! {
            #init_expr
            #(
                defmt::info!(#unit_test_running);
                #unit_test_calls
                defmt::info!(#unit_test_done);
            )*

            #krate::export::exit()
        }

        #init_fn

        #(
            fn #unit_test_names(#unit_test_inputs) #unit_test_blocks
        )*
    })
    .into())
}

#[derive(Clone, Copy)]
enum Attr {
    Init,
    Test,
}

struct Init {
    block: Box<Block>,
    ident: Ident,
    state: Option<Box<Type>>,
}

struct Test {
    block: Box<Block>,
    ident: Ident,
    input: Option<Input>,
}

struct Input {
    arg: FnArg,
    ty: Type,
}

fn path_is_ident(path: &Path, s: &str) -> bool {
    path.get_ident().map(|ident| ident == s).unwrap_or(false)
}

// NOTE doesn't check the parameters or the return type
fn check_fn_sig(sig: &syn::Signature) -> Result<(), ()> {
    if sig.constness.is_none()
        && sig.asyncness.is_none()
        && sig.unsafety.is_none()
        && sig.abi.is_none()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.variadic.is_none()
    {
        Ok(())
    } else {
        Err(())
    }
}

fn get_mutable_reference_type(arg: &syn::FnArg) -> Option<&Type> {
    if let syn::FnArg::Typed(pat) = arg {
        if let syn::Type::Reference(refty) = &*pat.ty {
            if refty.mutability.is_some() {
                Some(&refty.elem)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}
