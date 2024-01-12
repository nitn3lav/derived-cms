use std::{convert::Infallible, error::Error};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use thiserror::Error;

use crate::{context::ContextTrait, entity::EntityHooks, Entity};

#[derive(Debug, Error)]
pub enum ApiError<H: Error + Send> {
    #[error("Database error: {0}")]
    Database(#[from] ormlite::Error),
    #[error(transparent)]
    Hook(H),
}

impl<H: Error + Send> IntoResponse for ApiError<H> {
    fn into_response(self) -> Response {
        format!("{self:#}").into_response()
    }
}

pub async fn get_entities<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Query(filters): Query<Vec<(String, String)>>,
) -> Result<Json<Vec<E>>, ApiError<Infallible>> {
    let mut q = E::select();
    for (k, v) in filters {
        q = q.dangerous_where(&format!(
            "{} = {}",
            format_sql_query::Column((&*k).into()),
            format_sql_query::QuotedData(&v)
        ))
    }
    Ok(Json(q.fetch_all(ctx.db()).await?))
}

pub async fn get_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Path(id): Path<E::Id>,
) -> Result<Json<E>, ApiError<Infallible>> {
    Ok(Json(E::fetch_one(id, ctx.db()).await?))
}

/// create a new entity
pub async fn post_entities<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    Json(data): Json<E>,
) -> Result<Json<E>, ApiError<impl Error + Send>> {
    Ok(Json(
        data.on_create(ext)
            .await
            .map_err(ApiError::Hook)?
            .insert(ctx.db())
            .await?,
    ))
}

/// update existing entity
pub async fn post_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    Path(id): Path<E::Id>,
) -> Result<Json<E>, ApiError<Infallible>> {
    todo!()
}

pub async fn delete_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    Path(id): Path<E::Id>,
) -> Result<(), ApiError<impl Error + Send>> {
    let db = ctx.db();
    Ok(E::fetch_one(id, db)
        .await?
        .on_delete(ext)
        .await
        .map_err(ApiError::Hook)?
        .delete(db)
        .await?)
}
