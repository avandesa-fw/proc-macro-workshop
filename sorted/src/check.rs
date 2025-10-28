use crate::check_sorting::check_sorting;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::ExprMatch;

#[derive(Default)]
struct CheckSortedMatch {
    pub sorting_errors: TokenStream,
    pub non_sortable: TokenStream,
}

impl VisitMut for CheckSortedMatch {
    fn visit_expr_match_mut(&mut self, node: &mut ExprMatch) {
        // remove `#[sorted]` attributes, if present
        // todo: emit error for malformed attribute
        let _ = node
            .attrs
            .extract_if(.., |attr| attr.path().is_ident("sorted"))
            .map(|_| ())
            .collect::<()>();

        let idents = match node
            .arms
            .iter()
            .map(|arm| match &arm.pat {
                syn::Pat::Struct(pat_struct) => pat_struct.path.require_ident(),
                syn::Pat::TupleStruct(tuple_struct) => tuple_struct.path.require_ident(),
                _ => Err(syn::Error::new(arm.pat.span(), "can't sort this")),
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(idents) => idents,
            Err(e) => {
                self.non_sortable.extend(e.into_compile_error());
                return;
            }
        };

        if let Some(e) = check_sorting(idents) {
            self.sorting_errors.extend(e);
        }
    }
}

pub fn execute(_args: TokenStream, mut input: syn::Item) -> syn::Result<TokenStream> {
    let syn::Item::Fn(ref mut item_fn) = input else {
        return Err(syn::Error::new(Span::call_site(), "expected function"));
    };

    let mut checker = CheckSortedMatch::default();
    syn::visit_mut::visit_item_fn_mut(&mut checker, item_fn);

    let mut stream = input.into_token_stream();
    stream.extend(checker.sorting_errors);
    stream.extend(checker.non_sortable);

    Ok(stream)
}
