use axum::{
    routing::{get, post},
    Router,
};
use convert_case::{Case, Casing};
use generic_array::{ArrayLength, GenericArray};
use ormlite::Model;
use serde::{Deserialize, Serialize};

use crate::{column::Column, endpoints, input::InputInfo, render, DB};

pub use derived_cms_derive::Entity;

pub trait Entity:
    for<'de> Deserialize<'de>
    + Serialize
    + Model<DB>
    + for<'r> sqlx::FromRow<'r, <DB as sqlx::Database>::Row>
    + Send
    + Sync
    + Unpin
    + 'static
{
    /// should usually be an UUID
    type Id: for<'de> Deserialize<'de> + Serialize + Default + Send;

    type NumberOfColumns: ArrayLength;

    fn name() -> &'static str;
    fn name_plural() -> &'static str;

    fn column_names() -> GenericArray<&'static str, Self::NumberOfColumns>;
    fn column_values<'a>(&'a self) -> GenericArray<&'a dyn Column, Self::NumberOfColumns>;
    fn inputs(value: Option<&Self>) -> impl IntoIterator<Item = InputInfo<'_>>;

    fn routes<S: render::ContextTrait + 'static>() -> Router<S> {
        Router::new()
            .route(
                &format!("/{}", Self::name_plural().to_case(Case::Kebab)),
                get(endpoints::get_entities::<Self, S>),
            )
            .route(
                &format!("/{}/add", Self::name_plural().to_case(Case::Kebab)),
                get(endpoints::get_add_entity::<Self, S>),
            )
            .route(
                &format!("/{}/add", Self::name_plural().to_case(Case::Kebab)),
                post(endpoints::post_add_entity::<Self, S>),
            )
    }
}
