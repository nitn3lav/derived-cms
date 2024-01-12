use std::sync::Arc;

use axum::{
    extract::{Extension, Path, RawForm, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use tracing::{debug, error};

use crate::{context::ContextTrait, entity::EntityHooks, render, Entity};

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

pub async fn get_entities<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
) -> Result<impl IntoResponse, AppError> {
    let r = E::select().fetch_all(ctx.db()).await.map_err(|e| {
        AppError::new(
            fl!(
                i18n,
                "error-list-entities",
                "title",
                name = E::name_plural().to_case(Case::Title)
            ),
            fl!(i18n, "error-list-entities", "db", error = format!("{e:#}")),
        )
    })?;
    Ok(render::entity_list_page(ctx, &i18n, &r))
}

pub async fn get_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    Path(id): Path<E::Id>,
) -> Result<impl IntoResponse, AppError> {
    let r = E::fetch_one(id, ctx.db()).await.map_err(|e| {
        AppError::new(
            fl!(
                i18n,
                "error-show-entity",
                "title",
                name = E::name_plural().to_case(Case::Title)
            ),
            fl!(i18n, "error-show-entity", "db", error = format!("{e:#}")),
        )
    })?;
    Ok(render::entity_page(ctx, &i18n, Some(&r)))
}

pub async fn get_add_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
) -> impl IntoResponse {
    render::add_entity_page::<E>(ctx, &i18n, None)
}

pub async fn post_add_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    RawForm(form): RawForm,
) -> Result<impl IntoResponse, AppError> {
    let e = serde_qs::Config::new(5, false)
        .deserialize_bytes::<E>(&form)
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-create-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(
                    i18n,
                    "error-create-entity",
                    "parse-form",
                    error = format!("{e:#}")
                ),
            )
        })?;
    debug!(
        "Creating new {}: {}",
        E::name().to_case(Case::Title),
        serde_json::to_string(&e).unwrap()
    );
    let e = e
        .on_create(ext)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-create-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                format!("{e:#}"),
            )
        })?
        .insert(ctx.db())
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-create-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(i18n, "error-create-entity", "db", error = format!("{e:#}")),
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

pub async fn delete_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    Path(id): Path<E::Id>,
) -> Result<impl IntoResponse, AppError> {
    let db = ctx.db();
    E::fetch_one(id, db)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-delete-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(i18n, "error-delete-entity", "db", error = format!("{e:#}")),
            )
        })?
        .on_delete(ext)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-delete-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                format!("{e:#}"),
            )
        })?
        .delete(db)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-delete-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(i18n, "error-delete-entity", "db", error = format!("{e:#}")),
            )
        })?;
    Ok(Redirect::to(&format!(
        "/{}",
        E::name().to_case(Case::Kebab)
    )))
}
