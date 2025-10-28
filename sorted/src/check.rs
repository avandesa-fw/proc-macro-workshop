use crate::check_sorting::{check_sorting, SimplifiedPath, Sortable};
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
        let sorted_attrs = node
            .attrs
            .extract_if(.., |attr| attr.path().is_ident("sorted"))
            .map(|_| ())
            .count();
        if sorted_attrs == 0 {
            return;
        }

        let idents = match node
            .arms
            .iter()
            .map(|arm| {
                let sortable = match &arm.pat {
                    syn::Pat::Ident(pat_ident) => Sortable::Ident(&pat_ident.ident),
                    syn::Pat::Path(pat_path) => {
                        Sortable::Path(SimplifiedPath::try_from(&pat_path.path)?)
                    }
                    syn::Pat::Struct(pat_struct) => {
                        Sortable::Path(SimplifiedPath::try_from(&pat_struct.path)?)
                    }
                    syn::Pat::TupleStruct(tuple_struct) => {
                        Sortable::Path(SimplifiedPath::try_from(&tuple_struct.path)?)
                    }
                    syn::Pat::Wild(pat_wild) => Sortable::Wildcard(&pat_wild.underscore_token),
                    _ => return Err(syn::Error::new(arm.pat.span(), "unsupported by #[sorted]")),
                };
                Ok(sortable)
            })
            .collect::<Result<Vec<Sortable>, _>>()
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
