extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{
    parse, parse_macro_input, spanned::Spanned, Block, FnArg, Ident, Item,
    ItemMod, Path, ReturnType,  Type,
};

#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return parse::Error::new(Span::call_site(), "`#[test]` attribute takes no arguments")
            .to_compile_error()
            .into();
    }

    let module = parse_macro_input!(input as ItemMod);

    let items = if let Some(content) = module.content {
        content.1
    } else {
        return parse::Error::new(module.span(), "module must be inline (e.g. `mod foo {}`)")
            .to_compile_error()
            .into();
    };

    let mut init = None;
    let mut tests = vec![];
    for item in items {
        let span = item.span();
        match item {
            Item::Fn(f) => {
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
                    return parse::Error::new(
                        span,
                        "only attributes `#[test]` and `#[init]` are accepted",
                    )
                    .to_compile_error()
                    .into();
                };

                match attr {
                    Attr::Init => {
                        // TODO check `const` etc.
                        if !f.sig.inputs.is_empty() {
                            // TODO replace with error message
                            panic!()
                        }

                        let rty = match f.sig.output {
                            ReturnType::Default => None,
                            ReturnType::Type(.., ty) => Some(ty),
                        };

                        // TODO error message (more than one `#[init]`)
                        assert!(init.is_none());

                        init = Some(Init {
                            block: f.block,
                            ident: f.sig.ident,
                            rty: rty,
                        });
                    }

                    Attr::Test => {
                        assert_eq!(f.sig.output, ReturnType::Default);
                        let input = if f.sig.inputs.len() == 1 {
                            // TODO check `&mut T`
                            Some(f.sig.inputs[0].clone())
                        } else if f.sig.inputs.len() == 0 {
                            None
                        } else {
                            // TODO error message
                            panic!();
                        };
                        // TODO check `-> ()`

                        tests.push(Test {
                            block: f.block,
                            ident: f.sig.ident,
                            input,
                        })
                    }
                }
            }

            _ => {
                return parse::Error::new(item.span(), "only function items are allowed")
                    .to_compile_error()
                    .into();
            }
        }
    }

    let krate = format_ident!("defmt_test");
    let ident = module.ident;
    // TODO if no `init` then there should be no state
    let mut has_state = false;
    let (init_fn, init_expr) = if let Some(init) = init {
        let init_ident = init.ident;
        let init_block = init.block;
        let init_ty = init.rty;

        if init_ty.is_some() {
            has_state = true;
        }

        (
            Some(quote!(fn #init_ident() -> #init_ty #init_block)),
            Some(quote!(#[allow(dead_code)] let mut state = #init_ident();)),
        )
    } else {
        (None, None)
    };

    let mut unit_test_calls = vec![];
    for test in &tests {
        let ident = &test.ident;
        let call = if test.input.is_some() {
            if !has_state {
                panic!()
            }

            quote!(#ident(&mut state);)
        } else {
            quote!(#ident();)
        };
        unit_test_calls.push(call);
    }
    let unit_test_names = tests.iter().map(|test| &test.ident);
    let unit_test_inputs = tests.iter().map(|test| &test.input);
    let unit_test_blocks = tests.iter().map(|test| &test.block);
    let unit_test_running = tests
        .iter()
        .map(|test| format!("running {} ..", test.ident))
        .collect::<Vec<_>>();
    let unit_test_done = tests
        .iter()
        .map(|test| format!(".. {} ok", test.ident))
        .collect::<Vec<_>>();
    quote!(mod #ident {
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
    .into()
}

#[derive(Clone, Copy)]
enum Attr {
    Init,
    Test,
}

struct Init {
    block: Box<Block>,
    ident: Ident,
    rty: Option<Box<Type>>,
}

struct Test {
    block: Box<Block>,
    ident: Ident,
    input: Option<FnArg>,
}

fn path_is_ident(path: &Path, s: &str) -> bool {
    path.get_ident().map(|ident| ident == s).unwrap_or(false)
}
