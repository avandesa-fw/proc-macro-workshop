mod named_field;
mod util;

use named_field::NamedFieldData;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    derive_builder(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_builder(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;

    let builder_name = syn::Ident::new(&format!("{name}Builder"), name.span());

    let struct_fields = named_field::extract_from_derive_input(&input)?;

    let builder = builder(vis, &builder_name, &struct_fields);
    let builder_initializer = builder_initializer(name, &builder_name, &struct_fields);
    let builder_impl = builder_impl(name, &builder_name, &struct_fields);

    Ok(output(builder, builder_initializer, builder_impl))
}

fn output(
    builder: TokenStream,
    builder_initializer: TokenStream,
    build_impl: TokenStream,
) -> TokenStream {
    quote! {
        #builder
        #builder_initializer
        #build_impl
    }
}

fn builder(
    vis: &syn::Visibility,
    builder_name: &syn::Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let builder_fields = struct_fields.iter().map(NamedFieldData::as_optional_field);

    quote! {
        #vis struct #builder_name {
            #(#builder_fields),*
        }
    }
}

fn builder_initializer(
    name: &syn::Ident,
    builder_name: &syn::Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let initializers = struct_fields
        .iter()
        .map(NamedFieldData::as_field_initializer);

    quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#initializers),*
                }
            }
        }
    }
}

fn builder_impl(
    name: &syn::Ident,
    builder_name: &syn::Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let setters = struct_fields.iter().map(NamedFieldData::as_setter_fn);
    let build_fn = build_fn(name, struct_fields);

    quote! {
        impl #builder_name {
            #build_fn
            #(#setters)*
        }
    }
}

fn build_fn(name: &syn::Ident, struct_fields: &[NamedFieldData]) -> TokenStream {
    let fields = struct_fields.iter().map(NamedFieldData::as_unwrapped_field);

    quote! {
        pub fn build(&mut self) -> ::std::result::Result<#name, ::std::boxed::Box<dyn ::std::error::Error>> {
            ::std::result::Result::Ok(#name {
                #(#fields),*
            })
        }
    }
}
