use std::fmt::Write as _;

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, Index, Meta, NestedMeta, Type};

use crate::consts;

pub(crate) fn codegen(
    fields: &Fields,
    format_string: &mut String,
    patterns: &mut Vec<TokenStream2>,
) -> syn::Result<Vec<TokenStream2>> {
    let (fields, fields_are_named) = match fields {
        Fields::Named(named) => (&named.named, true),
        Fields::Unit => return Ok(vec![]),
        Fields::Unnamed(unnamed) => (&unnamed.unnamed, false),
    };

    if fields.is_empty() {
        return Ok(vec![]);
    }

    if fields_are_named {
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

        let format_opt = get_defmt_format_option(field)?;
        let ty = as_native_type(&field.ty).unwrap_or_else(|| consts::TYPE_FORMAT.to_string());
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| format_ident!("arg{}", index));

        if let Some(FormatOption::Debug2Format) = format_opt {
            stmts.push(quote!(defmt::export::fmt(&defmt::Debug2Format(&#ident))));
        } else if let Some(FormatOption::Display2Format) = format_opt {
            stmts.push(quote!(defmt::export::fmt(&defmt::Display2Format(&#ident))));
        } else if ty == consts::TYPE_FORMAT {
            stmts.push(quote!(defmt::export::fmt(#ident)));
        } else {
            let method = format_ident!("{}", ty);
            stmts.push(quote!(defmt::export::#method(#ident)));
        }

        if field.ident.is_some() {
            // Named field.
            write!(format_string, "{}: {{={}:?}}", ident, ty).ok();

            patterns.push(quote!( #ident ));
        } else {
            // Unnamed (tuple) field.
            write!(format_string, "{{={}}}", ty).ok();

            let index = Index::from(index);
            patterns.push(quote!( #index: #ident ));
        }
    }

    if fields_are_named {
        format_string.push_str(" }}");
    } else {
        format_string.push(')');
    }

    Ok(stmts)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum FormatOption {
    Debug2Format,
    Display2Format,
}

/// If the field has a valid defmt attribute (e.g. `#[defmt(Debug2Format)]`), returns `Ok(Some(FormatOption))`.
/// Returns `Err` if we can't parse a valid defmt attribute.
/// Returns `Ok(None)` if there are no `defmt` attributes on the field.
fn get_defmt_format_option(field: &Field) -> syn::Result<Option<FormatOption>> {
    use syn::Error;
    let attrs = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("defmt"))
        .map(|a| a.parse_meta())
        .collect::<syn::Result<Vec<_>>>()?;
    if attrs.len() == 0 {
        return Ok(None);
    } else if attrs.len() > 1 {
        return Err(Error::new_spanned(
            field,
            "multiple `defmt` attributes not supported",
        ));
    } // else attrs.len() == 1
    let attr = &attrs[0];
    let args = match attr {
        Meta::List(list) => &list.nested,
        bad => return Err(syn::Error::new_spanned(bad, "unrecognized attribute")),
    };
    if args.len() != 1 {
        return Err(syn::Error::new_spanned(
            attr,
            "expected 1 attribute argument",
        ));
    }
    let arg = match &args[0] {
        NestedMeta::Meta(Meta::Path(arg)) => arg,
        bad => {
            return Err(syn::Error::new_spanned(
                bad,
                "expected `Debug2Format` or `Display2Format`",
            ))
        }
    };
    if arg.is_ident("Debug2Format") {
        Ok(Some(FormatOption::Debug2Format))
    } else if arg.is_ident("Display2Format") {
        Ok(Some(FormatOption::Display2Format))
    } else {
        Err(syn::Error::new_spanned(
            arg,
            "expected `Debug2Format` or `Display2Format`",
        ))
    }
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
