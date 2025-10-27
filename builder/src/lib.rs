use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, Type, Visibility};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let vis = input.vis;

    let builder_name = Ident::new(&format!("{name}Builder"), name.span());

    let struct_fields = extract_named_field_data(input.data);

    let builder = builder(&vis, &builder_name, &struct_fields);
    let builder_initializer = builder_initializer(&name, &builder_name, &struct_fields);
    let builder_impl = builder_impl(&name, &builder_name, &struct_fields);

    output(builder, builder_initializer, builder_impl).into()
}

struct NamedFieldData {
    name: Ident,
    ty: Type,
}

fn extract_named_field_data(data: Data) -> Vec<NamedFieldData> {
    let Data::Struct(data_struct) = data else {
        panic!("expected struct");
    };
    let Fields::Named(struct_fields) = data_struct.fields else {
        panic!("expected non-tuple struct");
    };

    struct_fields
        .named
        .into_iter()
        .map(|field| NamedFieldData {
            name: field.ident.unwrap(),
            ty: field.ty,
        })
        .collect()
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
    vis: &Visibility,
    builder_name: &Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let builder_fields = struct_fields.iter().map(optionalize_field);

    quote! {
        #vis struct #builder_name {
            #(#builder_fields),*
        }
    }
}

fn builder_initializer(
    name: &Ident,
    builder_name: &Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let fields = struct_fields.iter().map(|field| {
        let name = &field.name;
        quote! { #name: ::std::option::Option::None }
    });

    quote! {
        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#fields),*
                }
            }
        }
    }
}

fn builder_impl(
    name: &Ident,
    builder_name: &Ident,
    struct_fields: &[NamedFieldData],
) -> TokenStream {
    let setters = struct_fields.iter().map(setter);
    let build_fn = build_fn(name, struct_fields);

    quote! {
        impl #builder_name {
            #build_fn
            #(#setters)*
        }
    }
}

fn build_fn(name: &Ident, struct_fields: &[NamedFieldData]) -> TokenStream {
    let fields = struct_fields.iter().map(|field| {
        let field_name = &field.name;
        quote! { #field_name: self.#field_name.clone().ok_or("field not set")? }
    });

    quote! {
        pub fn build(&mut self) -> ::std::result::Result<#name, Box<dyn ::std::error::Error>> {
            ::std::result::Result::Ok(#name {
                #(#fields),*
            })
        }
    }
}

fn setter(field: &NamedFieldData) -> TokenStream {
    let name = &field.name;
    let ty = &field.ty;
    quote! {
        pub fn #name(&mut self, #name: #ty) -> &mut Self {
            self.#name = ::std::option::Option::Some(#name);
            self
        }
    }
}

fn optionalize_field(field: &NamedFieldData) -> TokenStream {
    let name = &field.name;
    let field_ty = &field.ty;
    quote! { #name: ::std::option::Option<#field_ty> }
}
