use attribute_derive::FromAttr;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput};

fn found_crate() -> TokenStream {
    let found_crate = crate_name("derived-cms").expect("derived-cms is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( ::#ident )
        }
    }
}

#[proc_macro_derive(Entity, attributes(cms))]
pub fn derive_entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => {
            derive_entity_struct(&input, data).unwrap_or_else(syn::Error::into_compile_error)
        }
        _ => quote!(compile_error!("`Entity` can only be derived for `struct`s")),
    }
    .into()
}

fn derive_entity_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let found_crate = found_crate();

    let ident = &input.ident;

    // TODO: handle #[serde(rename, rename_all)]
    let name = input.ident.to_string();
    let name_plural = format!("{name}s");

    let properties = data
        .fields
        .iter()
        .map(|f| {
            let Some(ident) = &f.ident else {
                return quote!(compile_error!(
                    "`Entity` can only be derived for `struct`s with named fields"
                ));
            };
            // TODO: handle #[serde(rename)]
            quote! {
                #found_crate::property::PropertyInfo {
                    name: stringify!(#ident),
                    value: ::std::boxed::Box::new(::std::option::Option::map(value, |v| &v.#ident)),
                },
            }
        })
        .collect::<TokenStream>();
    let cols = data
        .fields
        .iter()
        .map(|field| EntityStructInnerAttr::from_attributes(&field.attrs).map(|attr| (field, attr)))
        .filter_ok(|(_field, attr)| !attr.skip_in_column)
        .collect::<Result<Vec<_>, _>>()?;
    let number_of_columns = Ident::new(&format!("U{}", cols.len()), Span::call_site());

    Ok(quote! {
        #[automatically_derived]
        impl Entity for #ident {
            type NumberOfColumns = #found_crate::derive::generic_array::typenum::#number_of_columns;

            fn name() -> impl ::std::fmt::Display + ::std::convert::AsRef<str> {
                #name
            }
            fn name_plural() -> impl ::std::fmt::Display + ::std::convert::AsRef<str> {
                #name_plural
            }

            fn render_column_values(&self) -> #found_crate::derive::generic_array::GenericArray<#found_crate::derive::maud::Markup, Self::NumberOfColumns> {
                todo!()
            }

            fn properties<'a>(value: ::std::option::Option<&'a Self>) -> impl ::std::iter::IntoIterator<Item = #found_crate::property::PropertyInfo<'a>> {
                [
                    #properties
                ]
            }
        }
    })
}

#[derive(FromAttr)]
#[attribute(ident = collection)]
#[attribute(error(missing_field = "`{field}` was not specified"))]
struct EntityStructInnerAttr {
    /// Do not display this field in list columns
    skip_in_column: bool,
}
