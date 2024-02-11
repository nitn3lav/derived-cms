use axum::{extract::Path, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

use crate::{context::ContextTrait, entity};

#[derive(Error)]
#[error(transparent)]
pub struct ApiError<T: Serialize>(#[from] T);

impl<T: Serialize> IntoResponse for ApiError<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self.0).into_response()
    }
}

pub async fn get_entities<E: entity::List<S>, S: ContextTrait>(
    ext: E::RequestExt,
) -> Result<Json<Vec<E>>, ApiError<E::Error>> {
    Ok(Json(E::list(ext).await?.into_iter().collect()))
}

pub async fn get_entity<E: entity::Get<S>, S: ContextTrait>(
    ext: E::RequestExt,
    Path(id): Path<E::Id>,
) -> Result<Json<E>, ApiError<E::Error>> {
    Ok(Json(E::get(&id, ext).await?))
}

/// create a new entity
pub async fn post_entities<E: entity::Create<S>, S: ContextTrait>(
    ext: E::RequestExt,
    Json(data): Json<E::Create>,
) -> Result<Json<E>, ApiError<E::Error>> {
    Ok(Json(E::create(data, ext).await?))
}

/// update existing entity
pub async fn post_entity<E: entity::Update<S>, S: ContextTrait>(
    ext: E::RequestExt,
    Path(id): Path<E::Id>,
    Json(data): Json<E::Update>,
) -> Result<Json<E>, ApiError<E::Error>> {
    Ok(Json(E::update(&id, data, ext).await?))
}

pub async fn delete_entity<E: entity::Delete<S>, S: ContextTrait>(
    ext: E::RequestExt,
    Path(id): Path<E::Id>,
) -> Result<(), ApiError<E::Error>> {
    Ok(E::delete(&id, ext).await?)
}
