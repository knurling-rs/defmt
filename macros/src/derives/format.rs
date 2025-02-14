use codegen::DefmtAttr;
use proc_macro::TokenStream;
use proc_macro_error2::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

mod codegen;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let defmt_attr = match input
        .attrs
        .iter()
        .find(|meta| meta.path().is_ident("defmt"))
        .cloned()
        .map(DefmtAttr::try_from)
    {
        Some(Ok(maybe_attr)) => Some(maybe_attr),
        Some(Err(err)) => return err.into_compile_error().into(),
        None => None,
    };

    if defmt_attr.is_some_and(|d| d.transparent) {
        return expand_transparent(input);
    }

    let ident = &input.ident;
    let encode_data = match &input.data {
        Data::Enum(data) => codegen::encode_enum_data(ident, data),
        Data::Struct(data) => codegen::encode_struct_data(ident, data),
        Data::Union(_) => abort_call_site!("`#[derive(Format)]` does not support unions"),
    };

    let codegen::EncodeData {
        format_tag,
        stmts,
        where_predicates,
    } = match encode_data {
        Ok(data) => data,
        Err(e) => return e.into_compile_error().into(),
    };

    let codegen::Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = codegen::Generics::codegen(&mut input.generics, where_predicates);

    quote!(
        impl #impl_generics defmt::Format for #ident #type_generics #where_clause {
            fn format(&self, f: defmt::Formatter) {
                defmt::unreachable!()
            }

            fn _format_tag() -> defmt::Str {
                #format_tag
            }

            fn _format_data(&self) {
                #(#stmts)*
            }
        }
    )
    .into()
}

pub(crate) fn expand_transparent(input: DeriveInput) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let call = match &input.data {
        Data::Enum(data) => {
            let match_arms = data.variants.iter().map(|v| {
                let variant_name = &v.ident;
                if v.fields.len() != 1 {
                    abort_call_site!("Transparent format can only be applied when all variants have exactly one field.")
                }
                let field = v.fields.members().next().unwrap();
                quote! {
                    Self::#variant_name{ #field: inner } => inner.format(f)
                }
            });
            quote! {
                match &self {
                    #( #match_arms, )*
                }
            }
        }
        Data::Struct(data) => {
            if !data.fields.len() == 1 {
                abort_call_site!(
                    "Transparent format can only be applied to structs with one field."
                );
            }
            let members = data.fields.members();
            quote! {
                #(self.#members.format(f));*
            }
        }
        Data::Union(_) => abort_call_site!("`#[derive(Format)]` does not support unions"),
    };

    quote! {
        impl #impl_generics defmt::Format for #ident #ty_generics #where_clause {
            fn format(&self, f: defmt::Formatter) {
                #call
            }
        }
    }
    .into()
}
