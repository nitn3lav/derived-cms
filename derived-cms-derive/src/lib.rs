use std::borrow::Cow;

use convert_case::{Case, Casing};
use darling::{FromAttributes, FromDeriveInput, FromField, FromMeta, FromVariant};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Type};

#[derive(Debug, FromAttributes)]
#[darling(attributes(cms, serde))]
struct EntityStructOptions {
    rename: Option<String>,
    rename_all: Option<RenameAll>,
    table: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(cms, serde))]
struct EntityFieldOptions {
    ident: Option<Ident>,
    ty: Type,
    /// Do not display this field in list columns
    #[darling(default)]
    skip_in_column: bool,
    #[darling(default)]
    skip_input: bool,
    rename: Option<String>,

    // TODO: find a solution that doesn't require specifying all serde args
    #[darling(default)]
    default: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cms, serde))]
struct PropertyEnumOptions {
    rename_all: Option<RenameAll>,
    tag: String,
    content: String,
}

#[derive(Debug, FromVariant)]
struct PropertyVariantOptions {
    rename: Option<String>,
}

#[derive(Clone, Copy, Debug, FromMeta)]
enum RenameAll {
    LowerCase,
    #[darling(rename = "UPPERCASE")]
    UpperCase,
    #[darling(rename = "PascalCase")]
    PascalCase,
    #[darling(rename = "camelCase")]
    CamelCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    #[darling(rename = "kebab-case")]
    KebabCase,
    #[darling(rename = "SCREAMING-SNAKE-CASE")]
    ScreamingKebabCase,
}

impl From<RenameAll> for Case {
    fn from(value: RenameAll) -> Self {
        match value {
            RenameAll::LowerCase => Case::Lower,
            RenameAll::UpperCase => Case::Upper,
            RenameAll::PascalCase => Case::Pascal,
            RenameAll::CamelCase => Case::Camel,
            RenameAll::SnakeCase => Case::Snake,
            RenameAll::ScreamingSnakeCase => Case::ScreamingSnake,
            RenameAll::KebabCase => Case::Kebab,
            RenameAll::ScreamingKebabCase => Case::UpperKebab,
        }
    }
}

fn found_crate() -> TokenStream {
    let found_crate = crate_name("derived-cms").expect("derived-cms is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => quote!(derived_cms),
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

    let struct_attr = EntityStructOptions::from_attributes(&input.attrs)?;
    let name = renamed_name(
        ident.to_string(),
        struct_attr.rename.as_ref(),
        Some(Case::Snake),
    );
    let name_plural = format!("{name}s");
    let table = renamed_name(
        name_plural.clone(),
        struct_attr.table.as_ref(),
        Some(Case::Snake),
    );

    let fields = data
        .fields
        .iter()
        .map(|f| EntityFieldOptions::from_field(f))
        .collect::<Result<Vec<_>, _>>()?;

    let cols = fields
        .iter()
        .clone()
        .filter(|attr| !attr.skip_in_column)
        .collect::<Vec<_>>();
    let number_of_columns = Ident::new(&format!("U{}", cols.len()), Span::call_site());

    let properties = properties_fn(&fields, &struct_attr);
    let (db_insert, where_clauses) = insert(&ident, &table, &fields)?;

    Ok(quote! {
        #[automatically_derived]
        impl<DB: #found_crate::derive::sqlx::Database> #found_crate::Entity<DB> for #ident
        where
            #where_clauses
        {
            type NumberOfColumns = #found_crate::derive::generic_array::typenum::#number_of_columns;

            fn name() -> &'static ::std::primitive::str {
                #name
            }
            fn name_plural() -> &'static ::std::primitive::str {
                #name_plural
            }

            fn render_column_values(&self) -> #found_crate::derive::generic_array::GenericArray<#found_crate::derive::maud::Markup, Self::NumberOfColumns> {
                todo!()
            }

            #properties
        }
        #db_insert
    })
}

fn properties_fn(fields: &[EntityFieldOptions], struct_attr: &EntityStructOptions) -> TokenStream {
    let found_crate = found_crate();
    let properties = fields
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
                #found_crate::property::PropertyInfo {
                    name: #name,
                    value: ::std::boxed::Box::new(::std::option::Option::map(value, |v| &v.#ident)),
                },
            }
        })
        .collect::<TokenStream>();
    quote! {
        fn properties<'a>(value: ::std::option::Option<&'a Self>) -> impl ::std::iter::IntoIterator<Item = #found_crate::property::PropertyInfo<'a>> {
            [#properties]
        }
    }
}

fn insert(
    ident: &Ident,
    table: &str,
    fields: &[EntityFieldOptions],
) -> syn::Result<(TokenStream, TokenStream)> {
    let found_crate = found_crate();
    let where_clauses = fields
        .iter()
        .map(|f| {
            let ty = &f.ty;
            quote! {
                #ty: Type<DB>,
                for<'e> #ty: Encode<'e, DB>,
            }
        })
        .collect::<TokenStream>();
    let idents = fields
        .into_iter()
        .map(|f| {
            let Some(ident) = &f.ident else {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "`Entity` can only be derived for `struct`s with named fields",
                ));
            };
            Ok(ident)
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let names = idents
        .iter()
        .map(|ident| ident.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let values = idents
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!("INSERT INTO {table} ({names}) VALUES ({values})");
    let bind = idents
        .iter()
        .map(|ident| {
            quote! {.bind(&self.#ident)}
        })
        .collect::<TokenStream>();

    Ok((
        quote! {
            #[automatically_derived]
            impl<DB: #found_crate::derive::sqlx::Database> #found_crate::entity::Insert<DB> for #ident
            where
                #where_clauses
            {
                async fn insert<'c, E>(&self, db: E) -> #found_crate::derive::sqlx::Result<()>
                where
                    E: #found_crate::derive::sqlx::Executor<'c, Database = DB>,
                    for<'q> <DB as #found_crate::derive::sqlx::database::HasArguments<'q>>::Arguments: #found_crate::derive::sqlx::IntoArguments<'q, DB>,
                {
                    #found_crate::derive::sqlx::query(#query)
                        #bind
                        .execute(db)
                        .await?;
                    #found_crate::derive::sqlx::Result::Ok(())
                }
            }
        },
        where_clauses,
    ))
}

#[proc_macro_derive(Property, attributes(cms))]
pub fn derive_property(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => derive_property_struct(&input, data),
        Data::Enum(data) => derive_property_enum(&input, data),
        _ => Ok(quote!(compile_error!(
            "`Entity` can only be derived for `struct`s and `enum`s"
        ))),
    }
    .unwrap_or_else(syn::Error::into_compile_error)
    .into()
}

fn derive_property_struct(_input: &DeriveInput, _data: &DataStruct) -> syn::Result<TokenStream> {
    todo!()
}

fn renamed_name<'a>(
    s: String,
    rename: Option<impl Into<Cow<'a, str>>>,
    rename_all: Option<impl Into<Case>>,
) -> Cow<'a, str> {
    rename.map(Into::into).unwrap_or_else(|| match rename_all {
        Some(case) => s.to_case(case.into()).into(),
        None => s.into(),
    })
}

fn derive_property_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let found_crate = found_crate();

    let ident = &input.ident;
    let attr = PropertyEnumOptions::from_derive_input(input)?;

    let x = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let variant_attr = PropertyVariantOptions::from_variant(v)?;

            let ident = &v.ident;
            let tag = &attr.tag;
            let content = &attr.content;

            let name_tag = quote!(&::std::format!("{}[{}]", name, #tag));
            let name_content = quote!(&::std::format!("{}[{}]", name, #content));
            let value = renamed_name(ident.to_string(), variant_attr.rename, attr.rename_all);

            let content_val = match v.fields {
                syn::Fields::Named(_) => todo!(),
                syn::Fields::Unnamed(ref fields) => {
                    let fields = &fields.unnamed;
                    let fields = fields
                        .iter()
                        .enumerate()
                        .map(|(i, _)| Ident::new(&format!("U{i}"), Span::call_site()))
                        .map(|i| quote!(#i,))
                        .collect::<TokenStream>();
                    Some(quote! {
                        match value {
                            ::std::option::Option::Some(Self::#ident(#fields)) => {
                                _selected_idx = #i;
                                ::std::option::Option::Some(#fields)
                            },
                            _ => ::std::option::Option::None,
                        }
                    })
                }
                syn::Fields::Unit => None,
            };
            let content_val = content_val
                .map(|content_val| {
                    quote! {
                        ::std::option::Option::Some(#found_crate::property::PropertyInfo {
                            name: #name_content,
                            value: ::std::boxed::Box::new(#content_val),
                        })
                    }
                })
                .unwrap_or(quote!(::std::option::Option::None));

            Ok(quote! {
                #found_crate::property::EnumVariant {
                    name: #name_tag,
                    value: #value,
                    content: #content_val,
                },
            })
        })
        .collect::<syn::Result<TokenStream>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl #found_crate::Property for #ident {
            fn render_input(
                value: Option<&Self>,
                name: &str,
                _name_human: &str,
                ctx: &derived_cms::render::FormRenderContext,
            ) -> #found_crate::derive::maud::Markup {
                let mut _selected_idx = 0;
                #found_crate::render::property_enum(&[#x], _selected_idx, ctx)
            }
        }
    })
}
