use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, FieldsNamed, Ident, Visibility};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let vis = input.vis;

    let builder_name = Ident::new(&format!("{name}Builder"), name.span());

    let Data::Struct(data_struct) = input.data else {
        panic!("expected struct");
    };
    let Fields::Named(struct_fields) = data_struct.fields else {
        panic!("expected non-tuple struct");
    };

    let builder = builder(&vis, &builder_name, &struct_fields);
    let builder_initializer = builder_initializer(&name, &builder_name, &struct_fields);
    let setters = setters(&builder_name, &struct_fields);

    output(builder, builder_initializer, setters).into()
}

fn output(
    builder: TokenStream,
    builder_initializer: TokenStream,
    setters: TokenStream,
) -> TokenStream {
    quote! {
        #builder
        #builder_initializer
        #setters
    }
}

fn builder(vis: &Visibility, builder_name: &Ident, struct_fields: &FieldsNamed) -> TokenStream {
    let builder_fields = struct_fields.named.iter().map(optionalize_field);

    quote! {
        #vis struct #builder_name {
            #(#builder_fields),*
        }
    }
}

fn builder_initializer(
    name: &Ident,
    builder_name: &Ident,
    struct_fields: &FieldsNamed,
) -> TokenStream {
    let fields = struct_fields.named.iter().map(|field| {
        let name = field.ident.as_ref().expect("named fields");
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

fn setters(builder_name: &Ident, struct_fields: &FieldsNamed) -> TokenStream {
    let setters = struct_fields.named.iter().map(setter);
    quote! {
        impl #builder_name {
            #(#setters)*
        }
    }
}

fn setter(field: &Field) -> TokenStream {
    let name = field.ident.as_ref().expect("named field");
    let ty = &field.ty;
    quote! {
        pub fn #name(&mut self, #name: #ty) -> &mut Self {
            self.#name = ::std::option::Option::Some(#name);
            self
        }
    }
}

fn optionalize_field(field: &Field) -> TokenStream {
    let name = field.ident.as_ref().expect("named field");
    let field_ty = &field.ty;
    quote! { #name: ::std::option::Option<#field_ty> }
}
