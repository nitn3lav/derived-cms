use std::sync::Arc;

use axum::{
    extract::{multipart::MultipartError, Multipart, Path, State},
    response::{IntoResponse, Redirect},
    Extension,
};
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{error, trace};
use uuid::Uuid;

use crate::{app::AppError, context::ContextTrait, entity, render, Entity};

pub async fn get_entities<E: Entity<S>, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    ext: <E as entity::List<S>>::RequestExt,
) -> Result<impl IntoResponse, AppError> {
    let r = E::list(ext).await.map_err(Into::into)?;
    Ok(render::entity_list_page(ctx, &i18n, r))
}

pub async fn get_entity<E: Entity<S>, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    ext: <E as entity::Get<S>>::RequestExt,
    Path(id): Path<E::Id>,
) -> Result<impl IntoResponse, AppError> {
    let e = E::get(&id, ext).await.map_err(Into::into)?;
    Ok(render::entity_page(ctx, &i18n, Some(&e)))
}

pub async fn get_add_entity<E: Entity<S>, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
) -> impl IntoResponse {
    render::add_entity_page::<E, S>(ctx, &i18n, None)
}

pub async fn post_add_entity<E: entity::Create<S>, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    ext: E::RequestExt,
    form: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let e = parse_form::<E::Create>(form, ctx.uploads_dir())
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
    let e = E::create(e, ext).await.map_err(Into::into)?;
    let uri = &format!(
        "/{}/{}",
        E::name().to_case(Case::Kebab),
        urlencoding::encode(&e.id().to_string())
    );
    Ok(Redirect::to(uri))
}

pub async fn post_entity<E: Entity<S>, S: ContextTrait>(
    ctx: State<S>,
    Extension(i18n): Extension<Arc<FluentLanguageLoader>>,
    ext: <E as entity::Update<S>>::RequestExt,
    Path(id): Path<E::Id>,
    form: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let e = parse_form::<E::Update>(form, ctx.uploads_dir())
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
    let e = E::update(&id, e, ext).await.map_err(Into::into)?;
    Ok(render::entity_page(ctx, &i18n, Some(&e)))
}

pub async fn delete_entity<E: entity::Delete<S>, S: ContextTrait>(
    ext: E::RequestExt,
    Path(id): Path<E::Id>,
) -> Result<impl IntoResponse, AppError> {
    E::delete(&id, ext).await.map_err(Into::into)?;
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
    #[error("Failed to deserialize: {serde:#}: {query_string}")]
    Deserialize {
        serde: serde_qs::Error,
        query_string: String,
    },
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
            Some(filename) if !filename.is_empty() => {
                let id = Uuid::new_v4();
                if filename.contains('/') {
                    return Err(ParseFormError::FilenameSlash(filename.to_string()));
                }
                let filename_escaped = urlencoding::encode(filename);
                if !qs.is_empty() {
                    qs.push_str("&");
                }
                qs.push_str(&format!("{name}[name]={filename_escaped}&{name}[id]={id}"));
                let path = files_dir.join(id.to_string());
                tokio::fs::create_dir_all(&path).await?;
                let path = path.join(filename);
                tokio::fs::File::create(path)
                    .await?
                    .write_all(&field.bytes().await?)
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
            _ => {}
        };
    }
    Ok(serde_qs::Config::new(5, false)
        .deserialize_str(&qs)
        .map_err(|e| ParseFormError::Deserialize {
            serde: e,
            query_string: qs,
        })?)
}
