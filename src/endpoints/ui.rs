use axum::{
    extract::{Path, RawForm, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use convert_case::{Case, Casing};
use tracing::{debug, error};

use crate::{render, Entity};

pub struct AppError {
    pub title: String,
    pub description: String,
}

impl AppError {
    fn new(title: String, description: String) -> Self {
        Self { title, description }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("{}: {}", self.title, self.description);
        (
            StatusCode::BAD_REQUEST,
            render::error_page(&self.title, &self.description),
        )
            .into_response()
    }
}

pub async fn get_entities<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
) -> Result<impl IntoResponse, AppError> {
    let r = E::select().fetch_all(ctx.db()).await.map_err(|e| {
        AppError::new(
            format!("Failed to list {}", E::name_plural().to_case(Case::Title)),
            format!("Database error: {e:#}"),
        )
    })?;
    Ok(render::entity_list_page(ctx, &r))
}

pub async fn get_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
    Path(id): Path<E::Id>,
) -> Result<impl IntoResponse, AppError> {
    let r = E::fetch_one(id, ctx.db()).await.map_err(|e| {
        AppError::new(
            format!("Failed to show {}", E::name_plural().to_case(Case::Title)),
            format!("Database error: {e:#}"),
        )
    })?;
    Ok(render::entity_page(ctx, Some(&r)))
}

pub async fn get_add_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
) -> impl IntoResponse {
    render::add_entity_page::<E>(ctx, None)
}

pub async fn post_add_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
    RawForm(form): RawForm,
) -> Result<impl IntoResponse, AppError> {
    let e = serde_qs::Config::new(5, false)
        .deserialize_bytes::<E>(&form)
        .map_err(|e| {
            AppError::new(
                format!("Failed to create new {}", E::name().to_case(Case::Title)),
                format!("Failed to parse form: {e:#}"),
            )
        })?;
    debug!(
        "Creating new {}: {}",
        E::name().to_case(Case::Title),
        serde_json::to_string(&e).unwrap()
    );
    let e = e
        .on_create()
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to create new {}", E::name().to_case(Case::Title)),
                format!("{e:#}"),
            )
        })?
        .insert(ctx.db())
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to create new {}", E::name().to_case(Case::Title)),
                format!("Database error: {e:#}"),
            )
        })?;
    debug!(
        "Created new {}: {}",
        E::name().to_case(Case::Title),
        serde_json::to_string(&e).unwrap()
    );

    let uri = &format!(
        "/{}/{}",
        E::name().to_case(Case::Kebab),
        urlencoding::encode(&e.id().to_string())
    );
    Ok(Redirect::to(uri))
}
