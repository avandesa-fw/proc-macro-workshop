use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Field, Fields, GenericArgument, Ident, PathArguments,
    Type, TypePath, Visibility,
};

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

#[derive(Debug)]
struct NamedFieldData {
    name: Ident,
    ty: Type,
    option_ty: Option<Type>,
}

impl NamedFieldData {
    fn inner_ty(&self) -> &Type {
        self.option_ty.as_ref().unwrap_or(&self.ty)
    }

    pub fn as_optional_field(&self) -> TokenStream {
        let name = &self.name;
        let ty = self.inner_ty();
        quote! { #name: ::std::option::Option<#ty> }
    }

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

    pub fn as_unwrapped_field(&self) -> TokenStream {
        let field_name = &self.name;
        if self.option_ty.is_some() {
            quote! { #field_name: self.#field_name.clone() }
        } else {
            quote! { #field_name: self.#field_name.clone().ok_or("field not set")? }
        }
    }
}

impl From<Field> for NamedFieldData {
    fn from(field: Field) -> Self {
        let option_ty = extract_option_ty(&field.ty);
        Self {
            name: field.ident.unwrap(),
            ty: field.ty,
            option_ty,
        }
    }
}

fn extract_option_ty(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath { path, .. }) = ty else {
        return None;
    };

    if path.segments.len() != 1 {
        return None;
    }
    let first_segment = path.segments.first()?;

    if first_segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(path_args) = &first_segment.arguments else {
        return None;
    };

    if path_args.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner_ty) = path_args.args.first()? else {
        return None;
    };

    Some(inner_ty.clone())
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
        .map(NamedFieldData::from)
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
    let builder_fields = struct_fields.iter().map(NamedFieldData::as_optional_field);

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
    let setters = struct_fields.iter().map(NamedFieldData::as_setter_fn);
    let build_fn = build_fn(name, struct_fields);

    quote! {
        impl #builder_name {
            #build_fn
            #(#setters)*
        }
    }
}

fn build_fn(name: &Ident, struct_fields: &[NamedFieldData]) -> TokenStream {
    let fields = struct_fields.iter().map(NamedFieldData::as_unwrapped_field);

    quote! {
        pub fn build(&mut self) -> ::std::result::Result<#name, Box<dyn ::std::error::Error>> {
            ::std::result::Result::Ok(#name {
                #(#fields),*
            })
        }
    }
}
