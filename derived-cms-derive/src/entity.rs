use convert_case::Case;
use darling::{FromAttributes, FromField};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataStruct, DeriveInput, Field, Type};

use crate::util::{found_crate, renamed_name, RenameAll};

#[derive(Debug, FromAttributes)]
#[darling(attributes(cms, serde))]
struct EntityStructOptions {
    #[darling(default)]
    hooks: bool,
    rename: Option<String>,
    rename_all: Option<RenameAll>,
}

#[derive(Debug, FromField)]
#[darling(attributes(cms, serde))]
struct EntityFieldOptions {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    id: bool,
    /// Do not display this field in list columns
    #[darling(default)]
    skip_column: bool,
    #[darling(default)]
    skip_input: bool,
    rename: Option<String>,
}

impl EntityFieldOptions {
    fn parse(f: &Field) -> Result<Self, darling::Error> {
        // TODO: allow overwriting options from serde with #[cms(...)]
        let attrs = f
            .attrs
            .to_owned()
            .into_iter()
            // filter serde fields
            .filter(|a| {
                let path = a.path();
                if !path.is_ident(&Ident::new("serde", Span::call_site())) {
                    return true;
                }
                if let syn::Meta::NameValue(v) = &a.meta {
                    return v.path.is_ident(&Ident::new("rename", Span::call_site()));
                }
                false
            })
            .collect();
        let f = Field {
            attrs,
            vis: f.vis.clone(),
            mutability: f.mutability.clone(),
            ident: f.ident.clone(),
            colon_token: f.colon_token,
            ty: f.ty.clone(),
        };
        Self::from_field(&f)
    }
}

pub fn derive_struct(input: &DeriveInput, data: &DataStruct) -> syn::Result<TokenStream> {
    let found_crate = found_crate();

    let ident = &input.ident;

    let struct_attr = EntityStructOptions::from_attributes(&input.attrs)?;
    let name = renamed_name(
        ident.to_string(),
        struct_attr.rename.as_ref(),
        Some(Case::Snake),
    );
    let name_plural = format!("{name}s");

    let fields = data
        .fields
        .iter()
        .map(EntityFieldOptions::parse)
        .collect::<Result<Vec<_>, _>>()?;

    let mut id_iter = fields
        .iter()
        .filter(|attr| attr.id)
        .map(|attr| (&attr.ident, &attr.ty));
    let Some((id_ident, id_type)) = id_iter.next() else {
        return Ok(quote!(compile_error!(
            "an Entity must have exactly one id. help: add `#[cms(id)]` to your id field"
        )));
    };
    let Some(id_ident) = id_ident else {
        return Ok(quote!(compile_error!(
            "`Entity` can only be derived for `struct`s with named fields"
        )));
    };
    if let Some(_) = id_iter.next() {
        return Ok(quote!(compile_error!(
            "An Entity can only have exactly one id"
        )));
    }

    let cols = fields
        .iter()
        .filter(|attr| !attr.skip_column)
        .collect::<Vec<_>>();
    let number_of_columns = Ident::new(&format!("U{}", cols.len()), Span::call_site());

    let inputs = inputs_fn(&fields, &struct_attr);
    let column_names = column_names_fn(&fields, &struct_attr);
    let column_values = column_values_fn(&fields);

    let hooks = if !struct_attr.hooks {
        quote! {
            #[automatically_derived]
            impl #found_crate::EntityHooks for #ident {}
        }
    } else {
        quote!()
    };

    Ok(quote! {
        #[automatically_derived]
        impl #found_crate::Entity for #ident
        where
            Self: #found_crate::derive::ormlite::Model<#found_crate::DB>
        {
            type Id = #id_type;

            type NumberOfColumns = #found_crate::derive::generic_array::typenum::#number_of_columns;

            fn name() -> &'static ::std::primitive::str {
                #name
            }
            fn name_plural() -> &'static ::std::primitive::str {
                #name_plural
            }

            fn id(&self) -> &#id_type {
                &self.#id_ident
            }

            #column_names
            #column_values
            #inputs
        }

        #hooks
    })
}

fn column_names_fn(
    fields: &[EntityFieldOptions],
    struct_attr: &EntityStructOptions,
) -> TokenStream {
    let found_crate = found_crate();
    let columns = fields
        .into_iter()
        .filter(|f| !f.skip_column)
        .map(|f| {
            let Some(ident) = &f.ident else {
                return quote!(compile_error!(
                    "`Entity` can only be derived for `struct`s with named fields"
                ));
            };
            let name = renamed_name(ident.to_string(), f.rename.as_ref(), struct_attr.rename_all);
            quote! {
                #name,
            }
        })
        .collect::<TokenStream>();
    quote! {
        fn column_names() -> #found_crate::derive::generic_array::GenericArray<&'static str, Self::NumberOfColumns> {
            #found_crate::derive::generic_array::arr![#columns]
        }
    }
}

fn column_values_fn(fields: &[EntityFieldOptions]) -> TokenStream {
    let found_crate = found_crate();
    let columns = fields
        .into_iter()
        .filter(|f| !f.skip_column)
        .map(|f| {
            let Some(ident) = &f.ident else {
                return quote!(compile_error!(
                    "`Entity` can only be derived for `struct`s with named fields"
                ));
            };
            quote! {
                &self.#ident,
            }
        })
        .collect::<TokenStream>();
    quote! {
        fn column_values<'a>(&'a self) -> #found_crate::derive::generic_array::GenericArray<&'a dyn #found_crate::Column, Self::NumberOfColumns> {
            #found_crate::derive::generic_array::arr![#columns]
        }
    }
}

fn inputs_fn(fields: &[EntityFieldOptions], struct_attr: &EntityStructOptions) -> TokenStream {
    let found_crate = found_crate();
    let inputs = fields
        .into_iter()
        .filter(|f| !f.skip_input)
        .map(|f| {
            let Some(ident) = &f.ident else {
                return quote!(compile_error!(
                    "`Entity` can only be derived for `struct`s with named fields"
                ));
            };
            let name = renamed_name(ident.to_string(), f.rename.as_ref(), struct_attr.rename_all);
            quote! {
                #found_crate::input::InputInfo {
                    name: #name,
                    value: ::std::boxed::Box::new(::std::option::Option::map(value, |v| &v.#ident)),
                },
            }
        })
        .collect::<TokenStream>();
    quote! {
        fn inputs<'a>(value: ::std::option::Option<&'a Self>) -> impl ::std::iter::IntoIterator<Item = #found_crate::input::InputInfo<'a>> {
            [#inputs]
        }
    }
}
