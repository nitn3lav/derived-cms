use std::{convert::Infallible, error::Error, fmt::Display, future::Future};

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
    EntityHooks
    + for<'de> Deserialize<'de>
    + Serialize
    + Model<DB>
    + for<'r> sqlx::FromRow<'r, <DB as sqlx::Database>::Row>
    + Send
    + Sync
    + Unpin
    + 'static
{
    /// should usually be an UUID
    type Id: for<'de> Deserialize<'de>
        + Display
        + Serialize
        + sqlx::Type<DB>
        + for<'q> sqlx::Encode<'q, DB>
        + Send;

    type NumberOfColumns: ArrayLength;

    fn name() -> &'static str;
    fn name_plural() -> &'static str;

    fn id(&self) -> &Self::Id;

    fn column_names() -> GenericArray<&'static str, Self::NumberOfColumns>;
    fn column_values<'a>(&'a self) -> GenericArray<&'a dyn Column, Self::NumberOfColumns>;
    fn inputs(value: Option<&Self>) -> impl IntoIterator<Item = InputInfo<'_>>;

    fn routes<S: render::ContextTrait + 'static>() -> Router<S> {
        let name = Self::name().to_case(Case::Kebab);
        let name = urlencoding::encode(&name);
        let name_pl = Self::name_plural().to_case(Case::Kebab);
        let name_pl = urlencoding::encode(&name_pl);

        Router::new()
            // API
            .route(
                &format!("/api/v1/{name_pl}"),
                get(endpoints::api::get_entities::<Self, S>),
            )
            .route(
                &format!("/api/v1/{name}/:id"),
                get(endpoints::api::get_entity::<Self, S>),
            )
            .route(
                &format!("/api/v1/{name_pl}"),
                post(endpoints::api::post_entities::<Self, S>),
            )
            .route(
                &format!("/api/v1/{name_pl}/:id"),
                post(endpoints::api::post_entity::<Self, S>),
            )
            // UI
            .route(
                &format!("/{name_pl}"),
                get(endpoints::ui::get_entities::<Self, S>),
            )
            .route(
                &format!("/{name}/:id"),
                get(endpoints::ui::get_entity::<Self, S>),
            )
            .route(
                &format!("/{name_pl}/add"),
                get(endpoints::ui::get_add_entity::<Self, S>),
            )
            .route(
                &format!("/{name_pl}/add"),
                post(endpoints::ui::post_add_entity::<Self, S>),
            )
    }
}

pub trait EntityHooks: Send + Sized {
    /// called before an [`Entity`] is inserted into the database
    fn on_create(self) -> impl Future<Output = Result<Self, impl Error + Send>> + Send {
        async { Result::<Self, Infallible>::Ok(self) }
    }

    /// called before an [`Entity`] is updated
    fn on_update(self) -> impl Future<Output = Result<Self, impl Error + Send>> + Send {
        async { Result::<Self, Infallible>::Ok(self) }
    }
}
