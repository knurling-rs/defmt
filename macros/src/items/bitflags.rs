use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::parse_macro_input;

use crate::{cargo, construct};

use self::input::Input;

mod input;

pub(crate) fn expand(input: TokenStream, is_v2: bool) -> TokenStream {
    let bitflags_input = TokenStream2::from(input.clone());
    let input = parse_macro_input!(input as Input);

    // Encode package and disambiguator to provide the decoder with all info it needs (even if
    // technically redundant, since it's also stored in the symbol we create).
    let format_string = format!(
        "{{={}:__internal_bitflags_{}@{}@{}@{}}}",
        input.ty().to_token_stream(),
        input.ident(),
        cargo::package_name(),
        construct::crate_local_disambiguator(),
        cargo::crate_name(),
    );
    let bitflags_tag = if is_v2 { "bitflagsv2" } else { "bitflags" };
    let format_tag = construct::interned_string(&format_string, bitflags_tag, false, None);

    let ident = input.ident();
    let ty = input.ty();
    let flag_statics = codegen_flag_statics(&input, is_v2);
    let bitflag_macro = format_ident!("{bitflags_tag}");
    quote!(
        const _: () = {
            fn assert<T: defmt::export::UnsignedInt>() {}
            assert::<#ty>;

            #(#flag_statics)*
        };

        defmt::export::#bitflag_macro! {
            #bitflags_input
        }

        impl defmt::Format for #ident {
            fn format(&self, f: defmt::Formatter) {
                defmt::unreachable!()
            }

            fn _format_tag() -> defmt::Str {
                #format_tag
            }

            fn _format_data(&self) {
                // There's a method available for every supported bitflags type.
                defmt::export::#ty(&self.bits());
            }
        }

    )
    .into()
}

fn codegen_flag_statics(input: &Input, is_v2: bool) -> Vec<TokenStream2> {
    input
        .flags()
        .enumerate()
        .map(|(i, flag)| {
            let cfg_attrs = flag.cfg_attrs();
            let var_name = flag.ident();
            let struct_name = input.ident();
            let repr_ty = input.ty();

            let _tag = if is_v2 {
                "bitflagsv2_value"
            } else {
                "bitflags_value"
            };
            let sym_name = construct::mangled_symbol_name(
                _tag,
                &format!("{}::{i}::{}", input.ident(), flag.ident()),
            );

            let bits_access = if is_v2 {
                quote! { bits() }
            } else {
                quote! { bits }
            };
            quote! {
                #(#cfg_attrs)*
                #[cfg_attr(target_os = "macos", link_section = ".defmt,end")]
                #[cfg_attr(not(target_os = "macos"), link_section = ".defmt.end")]
                #[export_name = #sym_name]
                static #var_name: u128 = {
                    // NB: It might be tempting to just do `#value as u128` here, but that
                    // causes a value such as `1 << 127` to be evaluated as an `i32`, which
                    // overflows. So we instead coerce (but don't cast) it to the bitflags' raw
                    // type, and then cast that to u128.
                    let coerced_value: #repr_ty = #struct_name::#var_name.#bits_access;
                    coerced_value as u128
                };
            }
        })
        .collect::<Vec<_>>()
}
