/// If the given type is `Option<T>`, returns `T`
pub fn extract_option_ty(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(syn::TypePath { path, .. }) = ty else {
        return None;
    };

    if path.segments.len() != 1 {
        return None;
    }
    let first_segment = path.segments.first()?;

    if first_segment.ident != "Option" {
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
