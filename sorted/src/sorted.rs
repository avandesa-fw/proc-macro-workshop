use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

pub fn execute(_args: TokenStream, input: syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Enum(ref _item_enum) = input else {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    Ok(input.into_token_stream())
}
