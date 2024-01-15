use std::{convert::Infallible, error::Error, fmt::Display};

use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlmo::query::Where;
use thiserror::Error;

use crate::{context::ContextTrait, entity::EntityHooks, Entity};

#[derive(Debug, Error)]
pub enum ApiError<H: Error + Send> {
    #[error("Database error: {0}")]
    Database(#[from] ormlite::Error),
    #[error(transparent)]
    Hook(H),
    #[error("{0}")]
    Other(String),
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
    let Where::And(ref mut w) = q.query.where_ else {
        return Err(ApiError::Other(
            "Select: `Where` was not `And`. This should never happen".to_string(),
        ));
    };
    for (k, v) in filters {
        let col = format_sql_query::Column((&*k).into());
        let mut or = vec![Where::Raw(format!(
            "{} = {}",
            col,
            format_sql_query::QuotedData(&v)
        ))];
        if let Some(v) = sql_literal(&v) {
            or.push(Where::Raw(format!("`{col}` = {v}")));
        };
        if let Some(v) = sql_binary(&v) {
            or.push(Where::Raw(format!("`{col}` = {v}")));
        }
        w.push(Where::Or(or));
    }
    Ok(Json(q.fetch_all(ctx.db()).await?))
}

fn sql_literal(s: &str) -> Option<&str> {
    (s.parse::<f64>().is_ok() || s.to_lowercase().parse::<bool>().is_ok()).then_some(s)
}

fn sql_binary(s: &str) -> Option<impl Display + '_> {
    struct R<'a>(&'a str);
    impl<'a> Display for R<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "X'{}'", self.0)
        }
    }
    ((s.len() & 1 == 0) && s.chars().all(|c| char::is_ascii_hexdigit(&c))).then_some(R(s))
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
    Json(new): Json<E>,
) -> Result<Json<E>, ApiError<impl Error + Send>> {
    let db = ctx.db();
    let old = E::fetch_one(id, db).await?;
    Ok(Json(
        E::on_update(old, new, ext)
            .await
            .map_err(ApiError::Hook)?
            .update_all_fields(db)
            .await?,
    ))
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
