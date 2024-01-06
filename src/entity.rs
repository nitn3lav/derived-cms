use std::future::Future;

use axum::{routing::get, Router};
use convert_case::{Case, Casing};
use generic_array::{ArrayLength, GenericArray};
use maud::Markup;
use serde::{Deserialize, Serialize};
use sqlx::Database;

use crate::{endpoints, property::PropertyInfo, render};

pub use derived_cms_derive::Entity;

pub trait Entity<DB: Database>: for<'de> Deserialize<'de> + Serialize + Insert<DB>
where
    Self: 'static,
{
    type NumberOfColumns: ArrayLength;

    fn name() -> &'static str;
    fn name_plural() -> &'static str;

    fn render_column_values(&self) -> GenericArray<Markup, Self::NumberOfColumns>;
    fn properties(value: Option<&Self>) -> impl IntoIterator<Item = PropertyInfo<'_>>;

    fn routes<S: render::ContextTrait<DB> + 'static>() -> Router<S> {
        Router::new().route(
            &format!("/{}/add", Self::name_plural().to_case(Case::Kebab)),
            get(endpoints::get_add_entity::<Self, DB, S>),
        )
    }
}

pub trait Insert<DB: Database> {
    fn insert<'c, E>(&self, db: E) -> impl Future<Output = sqlx::Result<()>>
    where
        E: sqlx::Executor<'c, Database = DB>,
        for<'q> <DB as sqlx::database::HasArguments<'q>>::Arguments: sqlx::IntoArguments<'q, DB>;
}
