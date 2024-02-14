use darling::{FromDeriveInput, FromField, FromVariant};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{DataEnum, DataStruct, DeriveInput, Field};

use crate::util::{found_crate, renamed_name, RenameAll};

/**********
 * struct *
 **********/

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cms, serde))]
struct InputStructOptions {
    rename_all: Option<RenameAll>,
}

#[derive(Debug, FromField)]
#[darling(attributes(cms, serde))]
struct InputFieldOptions {
    ident: Option<Ident>,
    /// Do not display this field in list columns
    #[darling(default)]
    skip_input: bool,
    rename: Option<String>,
}

impl InputFieldOptions {
    fn parse(f: &Field) -> Result<Self, darling::Error> {
        // TODO: allow overwriting options from serde with #[cms(...)]
        let attrs = f
            .attrs
            .iter()
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
            .cloned()
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
    let struct_attr = InputStructOptions::from_derive_input(input)?;

    let fields = data
        .fields
        .iter()
        .map(InputFieldOptions::parse)
        .collect::<Result<Vec<_>, _>>()?;

    let inputs = fields.iter().filter(|f| !f.skip_input).map(|f| {
        let Some(ident) = &f.ident else {
            return quote!(compile_error!(
                "`Entity` can only be derived for `struct`s with named fields"
            ));
        };
        let name = renamed_name(ident.to_string(), f.rename.as_ref(), struct_attr.rename_all);
        quote! {
            #found_crate::input::InputInfo {
                name: &::std::format!("{}[{}]", name, #name),
                name_human: #name,
                value: ::std::boxed::Box::new(::std::option::Option::map(value, |v| &v.#ident)),
            }
        }
    });

    Ok(quote! {
        #[automatically_derived]
        impl #found_crate::Input for #ident {
            fn render_input(
                value: ::std::option::Option<&Self>,
                name: &::std::primitive::str,
                _name_human: &::std::primitive::str,
                required: ::std::primitive::bool,
                ctx: &#found_crate::render::FormRenderContext,
                i18n: &#found_crate::derive::i18n_embed::fluent::FluentLanguageLoader,
            ) -> #found_crate::derive::maud::Markup {
                #found_crate::render::struct_input(ctx, i18n, [#(#inputs, )*])
            }
        }
    })
}

/********
 * enum *
 ********/

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

pub fn derive_enum(input: &DeriveInput, data: &DataEnum) -> syn::Result<TokenStream> {
    let found_crate = found_crate();

    let ident = &input.ident;
    let attr = InputEnumOptions::from_derive_input(input)?;

    let x = data
        .variants
        .iter()
        .map(|v| {
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
                            name_human: #content,
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

    let selected_idx = data.variants.iter().enumerate().map(|(i, v)| {
        let ident = &v.ident;
        let fields = match &v.fields {
            syn::Fields::Named(_) => quote!({ .. }),
            syn::Fields::Unnamed(f) => f
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    let ident = Ident::new(&format!("V{i}"), Span::call_site());
                    quote!((#ident,))
                })
                .collect(),
            syn::Fields::Unit => quote!(),
        };
        quote!(Self::#ident #fields => #i)
    });

    Ok(quote! {
        #[automatically_derived]
        impl #found_crate::Input for #ident {
            fn render_input(
                value: ::std::option::Option<&Self>,
                name: &::std::primitive::str,
                _name_human: &::std::primitive::str,
                required: ::std::primitive::bool,
                ctx: &#found_crate::render::FormRenderContext,
                i18n: &#found_crate::derive::i18n_embed::fluent::FluentLanguageLoader,
            ) -> #found_crate::derive::maud::Markup {
                let selected_idx = match value {
                    Some(v) => match v {
                        #(#selected_idx,)*
                    },
                    None => 0,
                };
                #found_crate::render::input_enum(ctx, i18n, &[#x], selected_idx, required)
            }
        }
    })
}
