use std::borrow::Cow;

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

#[derive(Clone, Copy, Debug, FromMeta)]
pub enum RenameAll {
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

pub fn found_crate() -> TokenStream {
    let found_crate = crate_name("derived-cms").expect("derived-cms is present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => quote!(derived_cms),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( ::#ident )
        }
    }
}

pub fn renamed_name<'a>(
    s: String,
    rename: Option<impl Into<Cow<'a, str>>>,
    rename_all: Option<impl Into<Case>>,
) -> Cow<'a, str> {
    rename.map(Into::into).unwrap_or_else(|| match rename_all {
        Some(case) => s.to_case(case.into()).into(),
        None => s.into(),
    })
}
