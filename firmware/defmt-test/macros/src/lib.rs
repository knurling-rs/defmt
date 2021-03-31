#![doc(html_logo_url = "https://knurling.ferrous-systems.com/knurling_logo_light_text.svg")]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse, spanned::Spanned, Attribute, Item, ItemFn, ItemMod, ReturnType, Type};

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
    let mut untouched_tokens = vec![];
    for item in items {
        match item {
            Item::Fn(mut f) => {
                let mut test_kind = None;
                let mut should_error = false;

                f.attrs.retain(|attr| {
                    if attr.path.is_ident("init") {
                        test_kind = Some(Attr::Init);
                        false
                    } else if attr.path.is_ident("test") {
                        test_kind = Some(Attr::Test);
                        false
                    } else if attr.path.is_ident("should_error") {
                        should_error = true;
                        false
                    } else {
                        true
                    }
                });

                let attr = match test_kind {
                    Some(it) => it,
                    None => {
                        return Err(parse::Error::new(
                            f.span(),
                            "function requires `#[init]` or `#[test]` attribute",
                        ));
                    }
                };

                match attr {
                    Attr::Init => {
                        if init.is_some() {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "only a single `#[init]` function can be defined",
                            ));
                        }

                        if should_error {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "`#[should_error]` is not allowed on the `#[init]` function",
                            ));
                        }

                        if check_fn_sig(&f.sig).is_err() || !f.sig.inputs.is_empty() {
                            return Err(parse::Error::new(
                                f.sig.ident.span(),
                                "`#[init]` function must have signature `fn() [-> Type]` (the return type is optional)",
                            ));
                        }

                        let state = match &f.sig.output {
                            ReturnType::Default => None,
                            ReturnType::Type(.., ty) => Some(ty.clone()),
                        };

                        init = Some(Init { func: f, state });
                    }

                    Attr::Test => {
                        if check_fn_sig(&f.sig).is_err() || f.sig.inputs.len() > 1 {
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
                                Some(Input { ty })
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
                            cfgs: extract_cfgs(&f.attrs),
                            func: f,
                            input,
                            should_error,
                        })
                    }
                }
            }

            _ => {
                untouched_tokens.push(item);
            }
        }
    }

    let krate = format_ident!("defmt_test");
    let ident = module.ident;
    let mut state_ty = None;
    let (init_fn, init_expr) = if let Some(init) = init {
        let init_func = &init.func;
        let init_ident = &init.func.sig.ident;
        state_ty = init.state;

        (
            Some(quote!(#init_func)),
            Some(quote!(#[allow(dead_code)] let mut state = #init_ident();)),
        )
    } else {
        (None, None)
    };

    let mut unit_test_calls = vec![];
    for test in &tests {
        let should_error = test.should_error;
        let ident = &test.func.sig.ident;
        let span = test.func.sig.ident.span();
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

            quote!(#ident(&mut state))
        } else {
            quote!(#ident())
        };
        unit_test_calls.push(quote!(
            #krate::export::check_outcome(#call, #should_error);
        ));
    }

    let test_functions = tests.iter().map(|test| &test.func);
    let test_cfgs = tests.iter().map(|test| &test.cfgs);
    let declare_test_count = {
        let test_cfgs = test_cfgs.clone();
        quote!(
            // We can't evaluate `#[cfg]`s in the macro, but this works too.
            const __DEFMT_TEST_COUNT: usize = {
                let mut counter = 0;
                #(
                    #(#test_cfgs)*
                    { counter += 1; }
                )*
                counter
            };
        )
    };
    let unit_test_running = tests
        .iter()
        .map(|test| {
            format!(
                "({{=usize}}/{{=usize}}) running `{}`...",
                test.func.sig.ident
            )
        })
        .collect::<Vec<_>>();
    Ok(quote!(mod #ident {
        #(#untouched_tokens)*
        // TODO use `cortex-m-rt::entry` here to get the `static mut` transform
        #[export_name = "main"]
        unsafe extern "C" fn __defmt_test_entry() -> ! {
            #declare_test_count
            #init_expr

            let mut __defmt_test_number: usize = 1;
            #(
                #(#test_cfgs)*
                {
                    defmt::info!(#unit_test_running, __defmt_test_number, __DEFMT_TEST_COUNT);
                    #unit_test_calls
                    __defmt_test_number += 1;
                }
            )*

            defmt::info!("all tests passed!");
            #krate::export::exit()
        }

        #init_fn

        #(
            #test_functions
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
    func: ItemFn,
    state: Option<Box<Type>>,
}

struct Test {
    func: ItemFn,
    cfgs: Vec<Attribute>,
    input: Option<Input>,
    should_error: bool,
}

struct Input {
    ty: Type,
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

fn extract_cfgs(attrs: &[Attribute]) -> Vec<Attribute> {
    let mut cfgs = vec![];

    for attr in attrs {
        if attr.path.is_ident("cfg") {
            cfgs.push(attr.clone());
        }
    }

    cfgs
}
