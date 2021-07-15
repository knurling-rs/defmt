use std::fmt::Write as _;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Fields, Index, Type};

use crate::consts;

pub(crate) fn codegen(
    fields: &Fields,
    format_string: &mut String,
    pats: &mut Vec<TokenStream2>,
) -> Vec<TokenStream2> {
    let (fields, named_fields) = match fields {
        Fields::Named(named) => (&named.named, true),
        Fields::Unit => return vec![],
        Fields::Unnamed(unnamed) => (&unnamed.unnamed, false),
    };

    if fields.is_empty() {
        return vec![];
    }

    if named_fields {
        format_string.push_str(" {{ ");
    } else {
        format_string.push('(');
    }

    let mut stmts = vec![];
    let mut is_first = true;
    for (index, field) in fields.iter().enumerate() {
        if is_first {
            is_first = false;
        } else {
            format_string.push_str(", ");
        }

        let ty = as_native_type(&field.ty).unwrap_or_else(|| consts::TYPE_FORMAT.to_string());
        if let Some(ident) = field.ident.as_ref() {
            write!(format_string, "{}: {{={}:?}}", ident, ty).ok();

            if ty == consts::TYPE_FORMAT {
                stmts.push(quote!(defmt::export::fmt(#ident)));
            } else {
                let method = format_ident!("{}", ty);
                stmts.push(quote!(defmt::export::#method(#ident)));
            }

            pats.push(quote!( #ident ));
        } else {
            // Unnamed (tuple) field.

            write!(format_string, "{{={}}}", ty).ok();

            let ident = format_ident!("arg{}", index);
            if ty == consts::TYPE_FORMAT {
                stmts.push(quote!(defmt::export::fmt(#ident)));
            } else {
                let method = format_ident!("{}", ty);
                stmts.push(quote!(defmt::export::#method(#ident)));
            }

            let index = Index::from(index);
            pats.push(quote!( #index: #ident ));
        }
    }

    if named_fields {
        format_string.push_str(" }}");
    } else {
        format_string.push(')');
    }

    stmts
}

/// Returns `Some` if `ty` refers to a builtin Rust type that has native support from defmt and does
/// not have to go through the `Format` trait.
///
/// This should return `Some` for all types that can be used as `{=TYPE}`.
///
/// Note: This is technically incorrect, since builtin types can be shadowed. However the efficiency
/// gains are too big to pass up, so we expect user code to not do that.
fn as_native_type(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => {
            let ident = path.path.get_ident()?;
            let ty_name = ident.to_string();

            match &*ty_name {
                "u8" | "u16" | "u32" | "usize" | "i8" | "i16" | "i32" | "isize" | "f32" | "f64"
                | "bool" | "str" => Some(ty_name),
                _ => None,
            }
        }
        Type::Reference(ty_ref) => as_native_type(&ty_ref.elem),
        _ => None,
    }
}
