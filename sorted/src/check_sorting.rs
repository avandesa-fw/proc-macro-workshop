use proc_macro2::{Span, TokenStream};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub enum Sortable<'ast> {
    Ident(&'ast syn::Ident),
}

impl Sortable<'_> {
    pub fn span(&self) -> Span {
        match self {
            Sortable::Ident(ident) => ident.span(),
        }
    }
}

impl PartialEq for Sortable<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ident(a), Self::Ident(b)) => a.eq(b),
        }
    }
}

impl PartialOrd for Sortable<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Ident(a), Self::Ident(b)) => a.partial_cmp(b),
        }
    }
}

impl Display for Sortable<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
        }
    }
}

pub fn check_sorting<'ast>(
    idents: impl IntoIterator<Item = Sortable<'ast>>,
) -> Option<TokenStream> {
    // initialize a map of each variant to the variant it should be sorted before
    // we'll populate it below, but each variant starts with `None`
    let map: Vec<(Rc<Sortable>, RefCell<Option<Rc<Sortable>>>)> = idents
        .into_iter()
        .map(|sortable| {
            let sortable = Rc::new(sortable);
            (sortable, RefCell::new(None))
        })
        .collect();

    // O(n^2) but who cares
    for (i, (a, _)) in map.iter().enumerate() {
        for (b, goes_before) in &map[i + 1..] {
            if a > b {
                let mut goes_before = goes_before.borrow_mut();
                // set `b_goes_before` if not set already
                // only override it if `a` is lexicographically before the existing value
                if goes_before.is_none()
                    || goes_before
                        .as_ref()
                        .is_some_and(|b_goes_before| a < &b_goes_before)
                {
                    *goes_before = Some(Rc::clone(a));
                }
            }
        }
    }

    // generate errors for any out-of-order ident and combine them into one token stream
    map.into_iter()
        .filter_map(|(b, goes_before)| {
            let a = goes_before.into_inner()?;
            Some(
                syn::Error::new(b.span(), format!("{b} should sort before {a}"))
                    .into_compile_error(),
            )
        })
        .reduce(|mut stream, error| {
            stream.extend(error);
            stream
        })
}
