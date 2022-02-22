use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

mod codegen;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let encode_data = match &input.data {
        Data::Enum(data) => codegen::encode_enum_data(ident, data),
        Data::Struct(data) => codegen::encode_struct_data(ident, data),
        Data::Union(_) => abort_call_site!("`#[derive(Format)]` does not support unions"),
    };

    let codegen::EncodeData { format_tag, stmts } = match encode_data {
        Ok(data) => data,
        Err(e) => return e.into_compile_error().into(),
    };

    let codegen::Generics {
        impl_generics,
        type_generics,
        where_clause,
    } = codegen::Generics::codegen(&mut input.generics);

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
