use syn::{GenericArgument, PathArguments, Type, TypePath};

/// If the given type is `Option<T>`, returns `T`
pub fn extract_option_ty(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };

    if path.segments.len() != 1 {
        return None;
    }
    let first_segment = path.segments.first()?;

    if first_segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(path_args) = &first_segment.arguments else {
        return None;
    };

    if path_args.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner_ty) = path_args.args.first()? else {
        return None;
    };

    Some(inner_ty.clone())
}
