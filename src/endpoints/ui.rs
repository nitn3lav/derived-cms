use std::sync::Arc;

use axum::{
    extract::{multipart::MultipartError, Extension, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};
use uuid::Uuid;

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
    form: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let e = parse_form::<E>(form, ctx.uploads_dir())
        .await
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

pub async fn post_entity<E: Entity, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    Extension(ext): Extension<<E as EntityHooks>::RequestExt<S>>,
    Path(id): Path<E::Id>,
    form: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let db = ctx.db();
    let mut new = parse_form::<E>(form, ctx.uploads_dir())
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-update-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(
                    i18n,
                    "error-update-entity",
                    "parse-form",
                    error = format!("{e:#}")
                ),
            )
        })?;
    new.set_id(id.clone());
    let old = E::fetch_one(id, db).await.map_err(|e| {
        AppError::new(
            fl!(
                i18n,
                "error-update-entity",
                "title",
                name = E::name().to_case(Case::Title)
            ),
            fl!(i18n, "error-update-entity", "db", error = format!("{e:#}")),
        )
    })?;
    let e = E::on_update(old, new, ext)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-update-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                format!("{e:#}"),
            )
        })?
        .update_all_fields(db)
        .await
        .map_err(|e| {
            AppError::new(
                fl!(
                    i18n,
                    "error-update-entity",
                    "title",
                    name = E::name().to_case(Case::Title)
                ),
                fl!(i18n, "error-update-entity", "db", error = format!("{e:#}")),
            )
        })?;
    Ok(render::entity_page(ctx, &i18n, Some(&e)))
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

#[derive(Debug, Error)]
enum ParseFormError {
    #[error("Multipart error: {0:#}")]
    Multipart(
        #[from]
        #[source]
        MultipartError,
    ),
    #[error("Field had no name")]
    NameMissing,
    #[error("Failed to write file: {0:#}")]
    Io(
        #[from]
        #[source]
        tokio::io::Error,
    ),
    #[error("File names must not contain '/': {0}")]
    FilenameSlash(String),
    #[error("Failed to deserialize: {0}")]
    Deserialize(
        #[from]
        #[source]
        serde_qs::Error,
    ),
}

/// Parse multipart/form-data with nested fields like in [serde_qs].
/// Files are stored with a unique id in `files_dir` and are deserialized as
/// ```rs
/// struct File {
///     /// name of the file created in `files_dir`
///     id: String,
///     /// original filename
///     name: String,
/// }
/// ```
async fn parse_form<T: for<'de> Deserialize<'de>>(
    mut form: Multipart,
    files_dir: &std::path::Path,
) -> Result<T, ParseFormError> {
    let mut qs = String::new();
    while let Some(field) = form.next_field().await? {
        let name = field.name().ok_or(ParseFormError::NameMissing)?;
        let name = urlencoding::encode(name);
        match field.file_name() {
            Some(filename) => {
                let id = Uuid::new_v4();
                if filename.contains('/') {
                    return Err(ParseFormError::FilenameSlash(filename.to_string()));
                }
                let new_filename = format!("{id}_{filename}");
                let filename = urlencoding::encode(filename);
                let id = urlencoding::encode(&new_filename);
                if !qs.is_empty() {
                    qs.push_str("&");
                }
                qs.push_str(&format!("{name}[name]={filename}&{name}[id]={id}"));
                tokio::fs::create_dir_all(&files_dir).await?;
                let path = files_dir.join(new_filename);
                tokio::fs::File::create(path)
                    .await?
                    .write(&field.bytes().await?)
                    .await?;
                // TODO: delete newly created files on error
            }
            None => {
                if !qs.is_empty() {
                    qs.push_str("&");
                }
                qs.push_str(&name);
                let bytes = field.bytes().await?;
                let value = urlencoding::encode_binary(&bytes);
                qs.push_str("=");
                qs.push_str(&value);
            }
        };
    }
    Ok(serde_qs::Config::new(5, false).deserialize_str(&qs)?)
}
