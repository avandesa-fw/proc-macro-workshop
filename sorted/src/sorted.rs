use proc_macro2::{Span, TokenStream};
use quote::ToTokens;

pub fn execute(_args: TokenStream, input: syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Enum(ref item_enum) = input else {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    for (i, a) in item_enum.variants.iter().enumerate() {
        for b in item_enum.variants.iter().skip(i + 1) {
            if a.ident > b.ident {
                return Err(syn::Error::new(
                    b.ident.span(),
                    format!("{} should sort before {}", b.ident, a.ident),
                ));
            }
        }
    }

    Ok(input.into_token_stream())
}
