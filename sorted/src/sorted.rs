use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::cell::RefCell;

pub fn execute(_args: TokenStream, input: syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Enum(ref item_enum) = input else {
        return Err(syn::Error::new(
            Span::call_site(),
            "expected enum or match expression",
        ));
    };

    // initialize a map of each variant to the variant it should be sorted before
    // we'll populate it below, but each variant starts with `None`
    let map: Vec<(&syn::Ident, RefCell<Option<&syn::Ident>>)> = item_enum
        .variants
        .iter()
        .map(|variant| (&variant.ident, RefCell::new(None)))
        .collect();

    // O(n^2) but who cares
    for (i, (a, _)) in map.iter().enumerate() {
        for (b, goes_before) in &map[i + 1..] {
            if a > b {
                let mut b_goes_before = goes_before.borrow_mut();
                // set `b_goes_before` if not set already
                // only override it if `a` is lexicographically before the existing value
                if b_goes_before.is_none()
                    || b_goes_before.is_some_and(|b_goes_before| a < &b_goes_before)
                {
                    *b_goes_before = Some(a);
                }
            }
        }
    }

    // generate errors for any out-of-order ident and combine them into one token stream
    let errors = map
        .into_iter()
        .filter_map(|(b, goes_before)| {
            let a = goes_before.into_inner()?;
            Some(
                syn::Error::new(b.span(), format!("{} should sort before {}", b, a))
                    .into_compile_error(),
            )
        })
        .reduce(|mut stream, error| {
            stream.extend(error);
            stream
        });

    let mut stream = input.into_token_stream();
    stream.extend(errors.into_iter().flatten());

    Ok(stream)
}
