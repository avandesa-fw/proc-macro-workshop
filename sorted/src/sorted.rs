use crate::check_sorting::check_sorting;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

pub fn execute(_args: TokenStream, input: syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Enum(ref item_enum) = input else {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    let errors = check_sorting(item_enum.variants.iter().map(|v| &v.ident));

    let mut stream = input.into_token_stream();
    stream.extend(errors.into_iter().flatten());

    Ok(stream)
}
