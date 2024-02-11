use std::{fmt::Display, future::Future};

use axum::extract::FromRequestParts;
use generic_array::{ArrayLength, GenericArray};
use serde::{Deserialize, Serialize};

use crate::{app::AppError, column::Column, context::ContextTrait, input::InputInfo};

pub use derived_cms_derive::Entity;

pub trait EntityBase<S: ContextTrait>:
    for<'de> Deserialize<'de> + Serialize + Send + Sync + Unpin + 'static
{
    /// should usually be an UUID
    type Id: for<'de> Deserialize<'de> + Clone + Display + Serialize + Send;

    type Create: for<'de> Deserialize<'de> + Serialize + Send + Sync + Unpin + 'static;
    type Update: for<'de> Deserialize<'de> + Serialize + Send + Sync + Unpin + 'static;

    type NumberOfColumns: ArrayLength;

    fn name() -> &'static str;
    fn name_plural() -> &'static str;

    /// should return the value of the field used as primary key.
    fn id(&self) -> &Self::Id;

    fn column_names() -> GenericArray<&'static str, Self::NumberOfColumns>;
    fn column_values(&self) -> GenericArray<&dyn Column, Self::NumberOfColumns>;
    fn inputs(value: Option<&Self>) -> impl IntoIterator<Item = InputInfo<'_>>;
}

pub trait Entity<S: ContextTrait>:
    EntityBase<S> + Get<S> + List<S> + Create<S> + Update<S> + Delete<S>
{
}

pub trait Get<S: ContextTrait>: EntityBase<S> {
    type RequestExt: FromRequestParts<S> + Send + Sync + Clone;
    type Error: Into<AppError> + Serialize + Send;

    fn get(
        id: &<Self as EntityBase<S>>::Id,
        ext: Self::RequestExt,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub trait List<S: ContextTrait>: EntityBase<S> {
    type RequestExt: FromRequestParts<S> + Send + Sync + Clone;
    type Error: Into<AppError> + Serialize + Send;

    // TODO: limit & offset
    fn list(
        ext: Self::RequestExt,
    ) -> impl Future<Output = Result<impl IntoIterator<Item = Self>, Self::Error>> + Send;
}

pub trait Create<S: ContextTrait>: EntityBase<S> {
    type RequestExt: FromRequestParts<S> + Send + Sync + Clone;
    type Error: Into<AppError> + Serialize + Send;

    fn create(
        data: <Self as EntityBase<S>>::Create,
        ext: Self::RequestExt,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub trait Update<S: ContextTrait>: EntityBase<S> {
    type RequestExt: FromRequestParts<S> + Send + Sync + Clone;
    type Error: Into<AppError> + Serialize + Send;

    fn update(
        id: &<Self as EntityBase<S>>::Id,
        data: <Self as EntityBase<S>>::Update,
        ext: Self::RequestExt,
    ) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

pub trait Delete<S: ContextTrait>: EntityBase<S> {
    type RequestExt: FromRequestParts<S> + Send + Sync + Clone;
    type Error: Into<AppError> + Serialize + Send + Sync + Unpin + 'static;

    fn delete(
        id: &<Self as EntityBase<S>>::Id,
        ext: Self::RequestExt,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
