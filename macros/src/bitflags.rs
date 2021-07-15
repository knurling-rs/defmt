use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse_macro_input;

use crate::{construct, symbol::Symbol};

pub(super) fn expand(ts: TokenStream) -> TokenStream {
    let ts2 = proc_macro2::TokenStream::from(ts.clone());
    let input: parser::Input = parse_macro_input!(ts);

    let codegen = Codegen::new(&input);

    // Encode package and disambiguator to provide the decoder with all info it needs (even if
    // technically redundant, since it's also stored in the symbol we create).
    let format_str = format!(
        "{{={}:__internal_bitflags_{}@{}@{}}}",
        input.ty().to_token_stream(),
        input.ident(),
        crate::symbol::package(),
        crate::symbol::disambiguator(),
    );
    let sym = construct::interned_string(&format_str, "bitflags", false);

    let ident = input.ident();
    let ty = input.ty();
    let flags = codegen.flags.iter().map(|f| &f.def);
    let result = quote! {
        const _: () = {
            fn assert<T: defmt::export::UnsignedInt>() {}
            assert::<#ty>;

            #(#flags)*
        };

        defmt::export::bitflags! {
            #ts2
        }

        impl defmt::Format for #ident {
            fn format(&self, f: defmt::Formatter) {
                unreachable!()
            }
            fn _format_tag() -> defmt::Str {
                #sym
            }
            fn _format_data(&self) {
                // There's a method available for every supported bitflags type.
                defmt::export::#ty(&self.bits());
            }
        }
    };
    result.into()
}

struct FlagStatic {
    def: TokenStream2,
}

struct Codegen {
    flags: Vec<FlagStatic>,
}

impl Codegen {
    fn new(input: &parser::Input) -> Self {
        let flags = input
            .flags()
            .map(|flag| {
                let cfg_attrs = flag.cfg_attrs();
                let name = &flag.ident();
                let value = &flag.value();
                let repr_ty = &input.ty();

                let sym = Symbol::new(
                    "bitflags_value",
                    &format!("{}::{}", input.ident(), flag.ident()),
                )
                .mangle();

                let def = quote! {
                    #(#cfg_attrs)*
                    #[cfg_attr(target_os = "macos", link_section = ".defmt,end")]
                    #[cfg_attr(not(target_os = "macos"), link_section = ".defmt.end")]
                    #[export_name = #sym]
                    static #name: u128 = {
                        // NB: It might be tempting to just do `#value as u128` here, but that
                        // causes a value such as `1 << 127` to be evaluated as an `i32`, which
                        // overflows. So we instead coerce (but don't cast) it to the bitflags' raw
                        // type, and then cast that to u128.
                        let coerced_value: #repr_ty = #value;
                        coerced_value as u128
                    };
                };
                FlagStatic { def }
            })
            .collect::<Vec<_>>();

        Codegen { flags }
    }
}

mod parser {
    use syn::{
        parse::{Parse, ParseStream},
        punctuated::Punctuated,
        token, Attribute, Expr, Ident, Token, Type, Visibility,
    };

    #[allow(dead_code)]
    pub(super) struct Input {
        struct_attrs: Vec<Attribute>,
        vis: Visibility,
        struct_token: Token![struct],
        ident: Ident,
        colon_token: Token![:],
        ty: Type,
        brace_token: token::Brace,
        flags: Punctuated<Flag, Token![;]>,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let flags;
            Ok(Self {
                struct_attrs: Attribute::parse_outer(input)?,
                vis: input.parse()?,
                struct_token: input.parse()?,
                ident: input.parse()?,
                colon_token: input.parse()?,
                ty: input.parse()?,
                brace_token: syn::braced!(flags in input),
                flags: Punctuated::parse_terminated(&flags)?,
            })
        }
    }

    impl Input {
        pub(super) fn flags(&self) -> impl Iterator<Item = &Flag> {
            self.flags.iter()
        }

        pub(super) fn ident(&self) -> &Ident {
            &self.ident
        }

        pub(super) fn ty(&self) -> &Type {
            &self.ty
        }
    }

    #[allow(dead_code)]
    pub(super) struct Flag {
        cfg_attrs: Vec<Attribute>,
        const_attrs: Vec<Attribute>,
        const_token: Token![const],
        ident: Ident,
        eq_token: Token![=],
        value: Expr,
    }

    impl Parse for Flag {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let const_attrs = Attribute::parse_outer(input)?;
            Ok(Self {
                cfg_attrs: extract_cfgs(&const_attrs),
                const_attrs,
                const_token: input.parse()?,
                ident: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        }
    }

    impl Flag {
        pub(super) fn cfg_attrs(&self) -> &[Attribute] {
            &self.cfg_attrs
        }

        pub(super) fn ident(&self) -> &Ident {
            &self.ident
        }

        pub(super) fn value(&self) -> &Expr {
            &self.value
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
}
