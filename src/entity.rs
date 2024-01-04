use axum::{routing::get, Router};
use convert_case::{Case, Casing};
use generic_array::{ArrayLength, GenericArray};
use maud::Markup;
use serde::{Deserialize, Serialize};

use crate::{property::PropertyInfo, render};

pub use derived_cms_derive::Entity;

pub trait Entity: for<'de> Deserialize<'de> + Serialize
where
    Self: 'static,
{
    type NumberOfColumns: ArrayLength;

    fn name() -> &'static str;
    fn name_plural() -> &'static str;

    fn render_column_values(&self) -> GenericArray<Markup, Self::NumberOfColumns>;
    fn properties(value: Option<&Self>) -> impl IntoIterator<Item = PropertyInfo<'_>>;

    fn routes() -> Router<render::Context> {
        Router::new().route(
            &format!("/{}/add", Self::name_plural().to_case(Case::Kebab)),
            get(render::add_entity_page::<Self>),
        )
    }
}
