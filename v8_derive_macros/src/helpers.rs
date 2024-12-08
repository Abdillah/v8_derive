use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn option_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(ty) = ty else { return None };
    if ty.qself.is_some() {
        return None;
    }

    let ty = &ty.path;

    if ty.segments.is_empty() || ty.segments.last().unwrap().ident != "Option" {
        return None;
    }

    if !(ty.segments.len() == 1
        || (ty.segments.len() == 3
            && ["core", "std"].contains(&ty.segments[0].ident.to_string().as_str())
            && ty.segments[1].ident == "option"))
    {
        return None;
    }

    let last_segment = ty.segments.last().unwrap();
    let syn::PathArguments::AngleBracketed(generics) = &last_segment.arguments else {
        return None;
    };
    if generics.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(inner_type) = &generics.args[0] else {
        return None;
    };

    Some(inner_type)
}

pub(crate) fn quote_get_field_as(
    ident: &syn::Ident,
    identifier: &syn::Ident,
    field: &syn::Field,
    optional: bool,
) -> Option<TokenStream> {
    let get_operation = if optional {
        quote! {
            v8_derive::get_optional_field_as
        }
    } else {
        quote! {
            v8_derive::get_field_as
        }
    };

    Some(if ident == "String" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_string)?
        }
    } else if ident == "bool" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_bool)?
        }
    } else if ident == "i32" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_i32)?
        }
    } else if ident == "i64" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_i64)?
        }
    } else if ident == "f64" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_f64)?
        }
    } else if ident == "f32" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_f32)?
        }
    } else if ident == "u32" {
        quote! {
            #get_operation(stringify!(#identifier), input, scope, v8_derive::helpers::try_as_u32)?
        }
    } else if ident == "Option" {
        let o_type = option_type(&field.ty)?;

        let syn::Type::Path(type_path) = o_type else {
            return None;
        };

        let ident = get_ident(type_path);
        quote_get_field_as(ident, identifier, field, true)?
    } else {
        return None;
    })
}

pub(crate) fn get_ident(type_path: &syn::TypePath) -> &syn::Ident {
    let path = &type_path.path;
    // todo: fix unwrap
    let segment = path.segments.first().unwrap();
    (&segment.ident) as _
}
