use std::borrow::Cow;

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

#[derive(Clone, Copy, Debug, FromMeta)]
pub enum RenameAll {
    Lower,
    #[darling(rename = "UPPERCASE")]
    Upper,
    #[darling(rename = "PascalCase")]
    Pascal,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "snake_case")]
    Snake,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
    #[darling(rename = "kebab-case")]
    Kebab,
    #[darling(rename = "SCREAMING-SNAKE-CASE")]
    ScreamingKebab,
}

impl From<RenameAll> for Case {
    fn from(value: RenameAll) -> Self {
        match value {
            RenameAll::Lower => Case::Lower,
            RenameAll::Upper => Case::Upper,
            RenameAll::Pascal => Case::Pascal,
            RenameAll::Camel => Case::Camel,
            RenameAll::Snake => Case::Snake,
            RenameAll::ScreamingSnake => Case::ScreamingSnake,
            RenameAll::Kebab => Case::Kebab,
            RenameAll::ScreamingKebab => Case::UpperKebab,
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
