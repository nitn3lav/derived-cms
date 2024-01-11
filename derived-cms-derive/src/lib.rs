use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};
use util::found_crate;

mod entity;
mod input;
mod util;

#[proc_macro_derive(Entity, attributes(cms))]
pub fn derive_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => {
            entity::derive_struct(&input, data).unwrap_or_else(syn::Error::into_compile_error)
        }
        _ => quote!(compile_error!("`Entity` can only be derived for `struct`s")),
    }
    .into()
}

#[proc_macro_derive(Input, attributes(cms))]
pub fn derive_input(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => input::derive_struct(&input, data),
        Data::Enum(data) => input::derive_enum(&input, data),
        _ => Ok(quote!(compile_error!(
            "`Entity` can only be derived for `struct`s and `enum`s"
        ))),
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

#[proc_macro_derive(Column, attributes(cms))]
pub fn derive_column(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let found_crate = found_crate();
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    quote! {
        impl Column for #ident {
            fn render(&self) -> #found_crate::derive::maud::Markup {
                #found_crate::derive::maud::html!((self))
            }
        }
    }
    .into()
}
