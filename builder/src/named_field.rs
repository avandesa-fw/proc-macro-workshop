use crate::util;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// Extract named struct field info from derive input data
pub fn extract_from_derive_data(data: syn::Data) -> Vec<NamedFieldData> {
    let syn::Data::Struct(data_struct) = data else {
        panic!("expected struct");
    };
    let syn::Fields::Named(struct_fields) = data_struct.fields else {
        panic!("expected non-tuple struct");
    };

    struct_fields
        .named
        .into_iter()
        .map(NamedFieldData::from)
        .collect()
}

#[derive(Debug)]
pub struct NamedFieldData {
    pub name: Ident,
    pub ty: syn::Type,
    pub option_ty: Option<syn::Type>,
}

impl NamedFieldData {
    /// If this field is `Option<T>`, then `T`, otherwise just the field's type
    fn inner_ty(&self) -> &syn::Type {
        self.option_ty.as_ref().unwrap_or(&self.ty)
    }

    /// Produce a struct field definition for this field, depending on whether it's an `Option`:
    /// `Option<T>` => `Option<T>`
    /// `T` => `Option<T>`
    pub fn as_optional_field(&self) -> TokenStream {
        let name = &self.name;
        let ty = self.inner_ty();
        quote! { #name: ::std::option::Option<#ty> }
    }

    /// Produce a setter function for this field on the builder
    ///
    /// ```rust
    /// pub fn this_field_name(&mut self, this_field_name: ThisFieldType) -> &mut Self {
    ///     self.this_field_name = Some(this_field_name);
    ///     self
    /// }
    pub fn as_setter_fn(&self) -> TokenStream {
        let name = &self.name;
        let ty = self.inner_ty();
        quote! {
            pub fn #name(&mut self, #name: #ty) -> &mut Self {
                self.#name = ::std::option::Option::Some(#name);
                self
            }
        }
    }

    /// Produce a struct field assignment, depending on whether it's an `Option`:
    /// `Option<T>` => `this_field_name: self.this_field_name.clone()`
    /// `T` => `this_field_name: self.this_field_name.clone().ok_or("field not set")?`
    pub fn as_unwrapped_field(&self) -> TokenStream {
        let field_name = &self.name;
        if self.option_ty.is_some() {
            quote! { #field_name: self.#field_name.clone() }
        } else {
            quote! { #field_name: self.#field_name.clone().ok_or("field not set")? }
        }
    }
}

impl From<syn::Field> for NamedFieldData {
    fn from(field: syn::Field) -> Self {
        let option_ty = util::extract_option_ty(&field.ty);
        Self {
            name: field.ident.unwrap(),
            ty: field.ty,
            option_ty,
        }
    }
}
