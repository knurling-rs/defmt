use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{DataEnum, Ident};

use crate::construct;

use super::EncodeData;

pub(crate) fn encode(
    ident: &Ident,
    data: &DataEnum,
    defmt_path: &syn::Path,
) -> syn::Result<EncodeData> {
    if data.variants.is_empty() {
        return Ok(EncodeData {
            stmts: vec![quote!(match *self {})],
            format_tag: construct::interned_string("!", "derived", false, None, defmt_path),
            where_predicates: vec![],
        });
    }

    let mut format_string = String::new();
    let mut where_predicates = vec![];

    let mut discriminant_arms = vec![];
    let mut match_arms = vec![];
    let mut has_variants_without_fields = false;
    let mut is_first_variant = true;
    let discriminant_encoder = DiscriminantEncoder::new(data.variants.len())?;
    let enum_ident = ident;
    for (index, variant) in data.variants.iter().enumerate() {
        let variant_ident = &variant.ident;

        if is_first_variant {
            is_first_variant = false;
        } else {
            format_string.push('|');
        }
        format_string.push_str(&variant_ident.to_string());

        let mut field_patterns = vec![];
        let (encode_fields_stmts, encode_field_where_predicates) = super::fields::codegen(
            &variant.fields,
            &mut format_string,
            &mut field_patterns,
            defmt_path,
        )?;
        where_predicates.extend(encode_field_where_predicates);
        let pattern = quote!( { #(#field_patterns),* } );

        if let Some(index) = discriminant_encoder.literal(index) {
            discriminant_arms.push(quote!(#enum_ident::#variant_ident { .. } => #index,));
        }

        if encode_fields_stmts.is_empty() {
            has_variants_without_fields = true;
        } else {
            match_arms.push(quote!(
                #enum_ident::#variant_ident #pattern => {
                    #(#encode_fields_stmts;)*
                }
            ))
        }
    }

    let format_tag = construct::interned_string(&format_string, "derived", false, None, defmt_path);
    let mut stmts = vec![];
    if let Some(method) = discriminant_encoder.method() {
        stmts.push(quote!(#defmt_path::export::#method(&match self {
            #(#discriminant_arms)*
        });));
    }
    if !match_arms.is_empty() {
        let default_arm = has_variants_without_fields.then(|| quote!(_ => {}));
        stmts.push(quote!(match self {
            #(#match_arms)*
            #default_arm
        }));
    }
    where_predicates.dedup_by(|a, b| a == b);

    Ok(EncodeData {
        format_tag,
        stmts,
        where_predicates,
    })
}

enum DiscriminantEncoder {
    Nop,
    U8,
    U16,
    U32,
    U64,
}

impl DiscriminantEncoder {
    // NOTE this assumes `number_of_variants` > 0
    fn new(number_of_variants: usize) -> syn::Result<Self> {
        // the width is picked by the largest discriminant, which must match the decoder
        let max_discriminant = number_of_variants - 1;
        if max_discriminant == 0 {
            Ok(Self::Nop)
        } else if u8::try_from(max_discriminant).is_ok() {
            Ok(Self::U8)
        } else if u16::try_from(max_discriminant).is_ok() {
            Ok(Self::U16)
        } else if u32::try_from(max_discriminant).is_ok() {
            Ok(Self::U32)
        } else if u64::try_from(max_discriminant).is_ok() {
            Ok(Self::U64)
        } else {
            // unreachable on existing hardware?
            Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "`#[derive(Format)]` does not support enums with more than {} variants",
                    number_of_variants
                ),
            ))
        }
    }

    fn method(&self) -> Option<Ident> {
        let name = match self {
            // For single-variant enums, there is no need to encode the discriminant.
            DiscriminantEncoder::Nop => return None,
            DiscriminantEncoder::U8 => "u8",
            DiscriminantEncoder::U16 => "u16",
            DiscriminantEncoder::U32 => "u32",
            DiscriminantEncoder::U64 => "u64",
        };
        Some(Ident::new(name, Span::call_site()))
    }

    // NOTE this assumes `index` < `number_of_variants` used to construct `self`
    fn literal(&self, index: usize) -> Option<TokenStream2> {
        match self {
            DiscriminantEncoder::Nop => None,
            DiscriminantEncoder::U8 => Some((index as u8).to_token_stream()),
            DiscriminantEncoder::U16 => Some((index as u16).to_token_stream()),
            DiscriminantEncoder::U32 => Some((index as u32).to_token_stream()),
            DiscriminantEncoder::U64 => Some((index as u64).to_token_stream()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiscriminantEncoder;

    #[test]
    fn discriminant_width() {
        let width = |n| match DiscriminantEncoder::new(n).unwrap() {
            DiscriminantEncoder::Nop => 0,
            DiscriminantEncoder::U8 => 1,
            DiscriminantEncoder::U16 => 2,
            DiscriminantEncoder::U32 => 4,
            DiscriminantEncoder::U64 => 8,
        };

        assert_eq!(width(1), 0);
        assert_eq!(width(2), 1);
        assert_eq!(width(256), 1);
        assert_eq!(width(257), 2);
        assert_eq!(width(65_536), 2);
        assert_eq!(width(65_537), 4);
    }
}
