use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::{render, Entity};

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] ormlite::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        format!("{self:#}").into_response()
    }
}

pub async fn get_entities<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
) -> Result<Json<Vec<E>>, ApiError> {
    Ok(Json(E::select().fetch_all(ctx.db()).await?))
}

pub async fn get_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
    Path(id): Path<E::Id>,
) -> Result<Json<E>, ApiError> {
    Ok(Json(E::fetch_one(id, ctx.db()).await?))
}

/// create a new entity
pub async fn post_entities<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
    Json(data): Json<E>,
) -> Result<Json<E>, ApiError> {
    data.clone().insert(ctx.db()).await?;
    Ok(Json(data))
}

/// update existing entity
pub async fn post_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
    Path(id): Path<E::Id>,
) -> Result<Json<E>, ApiError> {
    todo!()
}
