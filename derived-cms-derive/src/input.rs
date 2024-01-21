use darling::{FromDeriveInput, FromVariant};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataEnum, DataStruct, DeriveInput};

use crate::util::{found_crate, renamed_name, RenameAll};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cms, serde))]
struct InputEnumOptions {
    rename_all: Option<RenameAll>,
    tag: String,
    content: String,
}

#[derive(Debug, FromVariant)]
struct InputVariantOptions {
    rename: Option<String>,
}

pub fn derive_struct(_input: &DeriveInput, _data: &DataStruct) -> syn::Result<TokenStream> {
    todo!()
}

pub fn derive_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let found_crate = found_crate();

    let ident = &input.ident;
    let attr = InputEnumOptions::from_derive_input(input)?;

    let x = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let variant_attr = InputVariantOptions::from_variant(v)?;

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
                                selected_idx = #i;
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
                        ::std::option::Option::Some(#found_crate::input::InputInfo {
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
        impl #found_crate::Input for #ident {
            fn render_input(
                value: ::std::option::Option<&Self>,
                name: &::std::primitive::str,
                _name_human: &::std::primitive::str,
                required: ::std::primitive::bool,
                ctx: &derived_cms::render::FormRenderContext,
                i18n: &#found_crate::derive::i18n_embed::fluent::FluentLanguageLoader,
            ) -> #found_crate::derive::maud::Markup {
                let mut selected_idx = 0;
                #found_crate::render::input_enum(ctx, i18n, &[#x], selected_idx, required)
            }
        }
    })
}
