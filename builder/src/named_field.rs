use crate::util;
use proc_macro2::TokenStream;
use quote::quote;

/// Extract named struct field info from derive input data
pub fn extract_from_derive_data(data: syn::Data) -> syn::Result<Vec<NamedFieldData>> {
    let syn::Data::Struct(data_struct) = data else {
        panic!("expected struct");
    };
    let syn::Fields::Named(struct_fields) = data_struct.fields else {
        panic!("expected non-tuple struct");
    };

    struct_fields
        .named
        .into_iter()
        .map(NamedFieldData::try_from)
        .collect()
}

#[derive(Debug)]
pub enum NamedFieldKind {
    Normal,
    Option(syn::Type),
    VecWithEach(syn::Type, syn::Ident),
}

#[derive(Debug)]
pub struct NamedFieldData {
    pub name: syn::Ident,
    /// The bare type, may be `Option`, `Vec`, or something else
    pub ty: syn::Type,
    pub kind: NamedFieldKind,
}

impl NamedFieldData {
    /// If this field is `Option<T>` or `Vec<T>`, then `T`, otherwise just the field's type
    fn inner_ty(&self) -> &syn::Type {
        match &self.kind {
            NamedFieldKind::Normal => &self.ty,
            NamedFieldKind::Option(inner_ty) | NamedFieldKind::VecWithEach(inner_ty, _) => inner_ty,
        }
    }

    /// Produce a struct field definition for this field
    ///
    /// `Option<T>` => `Option<T>`
    /// `Vec<T>` => `Vec<T>`
    /// `T` => `Option<T>`
    pub fn as_optional_field(&self) -> TokenStream {
        let name = &self.name;
        match &self.kind {
            NamedFieldKind::Normal => {
                let ty = &self.ty;
                quote! { #name: ::std::option::Option<#ty> }
            }
            NamedFieldKind::Option(inner_ty) => quote! { #name: ::std::option::Option<#inner_ty> },
            NamedFieldKind::VecWithEach(inner_ty, _) => {
                quote! { #name: ::std::vec::Vec<#inner_ty> }
            }
        }
    }

    /// Produce an initializer for this field
    ///
    /// `Option<T>` => `this_field_name: None`
    /// `Vec<T>` => `this_field_name: Vec::new()`
    /// `T` => `this_field_name: None`
    pub fn as_field_initializer(&self) -> TokenStream {
        let name = &self.name;
        match &self.kind {
            NamedFieldKind::Normal | NamedFieldKind::Option(_) => {
                quote! { #name: ::std::option::Option::None }
            }
            NamedFieldKind::VecWithEach(_, _) => {
                quote! { #name: ::std::vec::Vec::new() }
            }
        }
    }

    /// Produce a setter function for this field on the builder
    ///
    /// ```ignore
    /// pub fn this_field_name(&mut self, this_field_name: ThisFieldType) -> &mut Self {
    ///     self.this_field_name = Some(this_field_name);
    ///     self
    /// }
    /// pub fn each_fn_name(&mut self, this_field_name: ThisFieldType) -> &mut Self {
    ///     self.this_field_name.push(this_field_name);
    ///     self
    /// }
    /// ```
    ///
    pub fn as_setter_fn(&self) -> TokenStream {
        let name = &self.name;
        let ty = self.inner_ty();

        match &self.kind {
            NamedFieldKind::Normal | NamedFieldKind::Option(_) => {
                quote! {
                    pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = ::std::option::Option::Some(#name);
                        self
                    }
                }
            }
            NamedFieldKind::VecWithEach(_, each_fn_name) => {
                quote! {
                    pub fn #each_fn_name(&mut self, #name: #ty) -> &mut Self {
                        self.#name.push(#name);
                        self
                    }
                }
            }
        }
    }

    /// Produce a struct field assignment
    ///
    /// `Option<T>` | `Vec<T>` => `this_field_name: self.this_field_name.clone()`
    /// `T` => `this_field_name: self.this_field_name.clone().ok_or("field not set")?`
    pub fn as_unwrapped_field(&self) -> TokenStream {
        let field_name = &self.name;
        match &self.kind {
            NamedFieldKind::Normal => {
                quote! { #field_name: self.#field_name.clone().ok_or("field not set")? }
            }
            NamedFieldKind::Option(_) | NamedFieldKind::VecWithEach(_, _) => {
                quote! { #field_name: self.#field_name.clone() }
            }
        }
    }
}

impl TryFrom<syn::Field> for NamedFieldData {
    type Error = syn::Error;
    fn try_from(field: syn::Field) -> Result<Self, Self::Error> {
        let kind = if let Some(each_fn_name) = util::extract_each_fn_name(&field.attrs)?
            && let Some(inner_ty) = util::extract_inner_ty(&field.ty, "Vec")
        {
            NamedFieldKind::VecWithEach(inner_ty, each_fn_name)
        } else if let Some(inner_ty) = util::extract_inner_ty(&field.ty, "Option") {
            NamedFieldKind::Option(inner_ty)
        } else {
            NamedFieldKind::Normal
        };

        Ok(Self {
            name: field.ident.unwrap(),
            ty: field.ty,
            kind,
        })
    }
}
