use codegen::DefmtAttr;
use proc_macro::TokenStream;
use proc_macro_error2::{abort, abort_call_site};
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Generics, Ident};

mod codegen;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let DeriveInput {
        attrs,
        vis: _,
        ident,
        mut generics,
        data,
    } = parse_macro_input!(input as DeriveInput);

    let DefmtAttr {
        transparent,
        defmt_path,
    } = match attrs
        .iter()
        .find(|meta| meta.path().is_ident("defmt"))
        .cloned()
        .map(DefmtAttr::try_from)
        .unwrap_or_else(|| Ok(DefmtAttr::default()))
    {
        Ok(maybe_attr) => maybe_attr,
        Err(err) => abort!(err),
    };

    if transparent {
        return expand_transparent(ident, data, generics, defmt_path);
    }

    let encode_data = match &data {
        Data::Enum(data) => codegen::encode_enum_data(&ident, data, &defmt_path),
        Data::Struct(data) => codegen::encode_struct_data(&ident, data, &defmt_path),
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
    } = codegen::Generics::codegen(&mut generics, where_predicates);

    quote!(
        #[automatically_derived]
        impl #impl_generics #defmt_path::Format for #ident #type_generics #where_clause {
            fn format(&self, f: #defmt_path::Formatter) {
                use #defmt_path as defmt;
                #defmt_path::unreachable!()
            }

            fn _format_tag() -> #defmt_path::Str {
                #format_tag
            }

            fn _format_data(&self) {
                #(#stmts)*
            }
        }
    )
    .into()
}

pub(crate) fn expand_transparent(
    ident: Ident,
    data: Data,
    mut generics: Generics,
    defmt_path: syn::Path,
) -> TokenStream {
    let mut where_clause = generics.make_where_clause().clone();
    let (impl_generics, ty_generics, ..) = generics.split_for_impl();

    let mut member_types: Vec<syn::Type> = vec![];
    let body = match data {
        Data::Enum(data) => {
            let match_arms = data.variants.iter().map(|v| -> syn::Arm {
                let variant_name = &v.ident;
                if v.fields.len() != 1 {
                    abort_call_site!("Transparent format can only be applied when all variants have exactly one field.")
                }

                member_types.push(v.fields.iter().next().unwrap().ty.clone());
                let field = v.fields.members().next().unwrap();
                parse_quote! {
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

            member_types.push(data.fields.iter().next().unwrap().ty.clone());
            let members = data.fields.members();
            quote! {
                #(self.#members.format(f));*
            }
        }
        Data::Union(_) => abort_call_site!("`#[derive(Format)]` does not support unions"),
    };

    let generic_bounds: Vec<syn::WherePredicate> = member_types
        .iter()
        .map(|ty| parse_quote! { #ty: #defmt_path::Format })
        .collect();
    where_clause.predicates.extend(generic_bounds);

    quote! {
        impl #impl_generics #defmt_path::Format for #ident #ty_generics #where_clause {
            fn format(&self, f: #defmt_path::Formatter) {
                #body
            }
        }
    }
    .into()
}
