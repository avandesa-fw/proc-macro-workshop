use proc_macro2::{Span, TokenStream};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use syn::spanned::Spanned;

pub struct SimplifiedPath<'ast> {
    pub source: &'ast syn::Path,
    pub stringified: String,
}

impl<'ast> TryFrom<&'ast syn::Path> for SimplifiedPath<'ast> {
    type Error = syn::Error;
    fn try_from(path: &'ast syn::Path) -> Result<Self, syn::Error> {
        if path.leading_colon.is_some() {
            return Err(syn::Error::new(
                path.leading_colon.span(),
                "unsupported by #[sorted]",
            ));
        }
        let mut stringified = String::new();
        for segment in &path.segments {
            if !matches!(segment.arguments, syn::PathArguments::None) {
                return Err(syn::Error::new(
                    segment.arguments.span(),
                    "unsupported by #[sorted]",
                ));
            }

            stringified = if stringified.is_empty() {
                segment.ident.to_string()
            } else {
                format!("{stringified}::{}", segment.ident)
            };
        }

        Ok(Self {
            source: path,
            stringified,
        })
    }
}

pub enum Sortable<'ast> {
    Ident(&'ast syn::Ident),
    Path(SimplifiedPath<'ast>),
    Wildcard(&'ast syn::Token![_]),
}

impl Sortable<'_> {
    pub fn span(&self) -> Span {
        match self {
            Sortable::Ident(ident) => ident.span(),
            Sortable::Path(path) => path.source.span(),
            Sortable::Wildcard(wildcard) => wildcard.span,
        }
    }
}

impl PartialEq for Sortable<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ident(a), Self::Ident(b)) => a.eq(b),
            (Self::Path(a), Self::Path(b)) => a.stringified.eq(&b.stringified),
            (Self::Wildcard(_), Self::Wildcard(_)) => true,
            _ => false,
        }
    }
}

impl PartialOrd for Sortable<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Ident(a), Self::Ident(b)) => a.partial_cmp(b),
            (Self::Path(a), Self::Path(b)) => a.stringified.partial_cmp(&b.stringified),
            (Self::Ident(ident), Self::Path(path)) => {
                ident.to_string().partial_cmp(&path.stringified)
            }
            (Self::Path(path), Self::Ident(ident)) => {
                path.stringified.partial_cmp(&ident.to_string())
            }
            (Self::Wildcard(_), Self::Wildcard(_)) => Some(Ordering::Equal),
            (Self::Wildcard(_), _) => Some(Ordering::Greater),
            (_, Self::Wildcard(_)) => Some(Ordering::Less),
        }
    }
}

impl Display for Sortable<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(ident) => ident.fmt(f),
            Sortable::Path(path) => path.stringified.fmt(f),
            Sortable::Wildcard(_) => "wildcard".fmt(f),
        }
    }
}

pub fn check_sorting<'ast>(
    idents: impl IntoIterator<Item = Sortable<'ast>>,
) -> Option<TokenStream> {
    // initialize a map of each variant to the variant it should be sorted before
    // we'll populate it below, but each variant starts with `None`
    // Vec<(Rc<Sortable>, RefCell<Option<Rc<Sortable>>>)>
    let map = idents
        .into_iter()
        .map(|sortable| {
            let sortable = Rc::new(sortable);
            (sortable, RefCell::new(None))
        })
        .collect::<Vec<_>>();

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
                        .is_some_and(|goes_before| a < goes_before)
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
