use std::convert::TryFrom;

use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{
    parse_quote, DataEnum, DataStruct, GenericParam, Ident, ImplGenerics, TypeGenerics, WhereClause,
};

pub(crate) struct EncodeData {
    pub(crate) format_tag: TokenStream2,
    pub(crate) stmts: Vec<TokenStream2>,
}

pub(crate) fn encode_struct_data(ident: &Ident, data: &DataStruct) -> EncodeData {
    let mut format_string = ident.to_string();
    let mut stmts = vec![];
    let mut patterns = vec![];

    let encode_field_stmts =
        crate::fields(&data.fields, &mut format_string, &mut vec![], &mut patterns);

    stmts.push(quote!(match self {
        Self { #(#patterns),* } => {
            #(#encode_field_stmts;)*
        }
    }));

    let format_tag = crate::mksym(&format_string, "derived", false);
    EncodeData { format_tag, stmts }
}

pub(crate) fn encode_enum_data(ident: &Ident, data: &DataEnum) -> EncodeData {
    if data.variants.is_empty() {
        return EncodeData {
            stmts: vec![quote!(match *self {})],
            format_tag: crate::mksym("!", "derived", false),
        };
    }

    let mut stmts = vec![];
    let mut field_types = vec![];
    let mut format_string = String::new();

    let mut arms = vec![];
    let mut first = true;
    for (i, var) in data.variants.iter().enumerate() {
        let vident = &var.ident;

        if first {
            first = false;
        } else {
            format_string.push('|');
        }
        format_string.push_str(&vident.to_string());

        let mut pats = vec![];
        let encode_fields_stmts =
            crate::fields(&var.fields, &mut format_string, &mut field_types, &mut pats);
        let pats = quote!( { #(#pats),* } );

        let len = data.variants.len();
        let encode_discriminant = if len == 1 {
            // For single-variant enums, there is no need to encode the discriminant.
            quote!()
        } else if let (Ok(_), Ok(i)) = (u8::try_from(len), u8::try_from(i)) {
            quote!(
                defmt::export::u8(&#i);
            )
        } else if let (Ok(_), Ok(i)) = (u16::try_from(len), u16::try_from(i)) {
            quote!(
                defmt::export::u16(&#i);
            )
        } else if let (Ok(_), Ok(i)) = (u32::try_from(len), u32::try_from(i)) {
            quote!(
                defmt::export::u32(&#i);
            )
        } else if let (Ok(_), Ok(i)) = (u64::try_from(len), u64::try_from(i)) {
            quote!(
                defmt::export::u64(&#i);
            )
        } else {
            // u128 case is omitted with the assumption, that usize is never greater than u64
            abort_call_site!(
                "`#[derive(Format)]` does not support enums with more than {} variants",
                u64::MAX
            )
        };

        arms.push(quote!(
            #ident::#vident #pats => {
                #encode_discriminant
                #(#encode_fields_stmts;)*
            }
        ))
    }

    stmts.push(quote!(match self {
        #(#arms)*
    }));

    let format_tag = crate::mksym(&format_string, "derived", false);

    EncodeData { format_tag, stmts }
}

pub(crate) struct Generics<'a> {
    pub(crate) impl_generics: ImplGenerics<'a>,
    pub(crate) type_generics: TypeGenerics<'a>,
    pub(crate) where_clause: WhereClause,
}

impl<'a> Generics<'a> {
    pub(crate) fn codegen(generics: &'a mut syn::Generics) -> Self {
        let mut where_clause = generics.make_where_clause().clone();
        let (impl_generics, type_generics, _) = generics.split_for_impl();

        // Extend where-clause with `Format` bounds for type parameters.
        for param in &generics.params {
            if let GenericParam::Type(ty) = param {
                let ident = &ty.ident;

                where_clause
                    .predicates
                    .push(parse_quote!(#ident: defmt::Format));
            }
        }

        Self {
            impl_generics,
            type_generics,
            where_clause,
        }
    }
}
