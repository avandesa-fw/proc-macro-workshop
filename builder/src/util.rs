/// If the given type is `outer_ty<T>`, returns `T`
pub fn extract_inner_ty(ty: &syn::Type, outer_ty: &str) -> Option<syn::Type> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else {
        return None;
    };

    if path.segments.len() != 1 {
        return None;
    }
    let first_segment = path.segments.first()?;

    if first_segment.ident != outer_ty {
        return None;
    }

    let syn::PathArguments::AngleBracketed(path_args) = &first_segment.arguments else {
        return None;
    };

    if path_args.args.len() != 1 {
        return None;
    }
    let syn::GenericArgument::Type(inner_ty) = path_args.args.first()? else {
        return None;
    };

    Some(inner_ty.clone())
}

pub fn extract_each_fn_name(attrs: &[syn::Attribute]) -> syn::Result<Option<syn::Ident>> {
    for attr in attrs {
        if !attr.path().is_ident("builder") {
            continue;
        }

        let mut each_fn_name = None;
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("each") {
                return Err(meta.error("expected `builder(each = \"...\")`"));
            }

            let lit = meta.value()?.parse::<syn::LitStr>()?;
            each_fn_name = Some(syn::Ident::new(&lit.value(), lit.span()));

            Ok(())
        })?;

        if each_fn_name.is_some() {
            return Ok(each_fn_name);
        }
    }

    Ok(None)
}
