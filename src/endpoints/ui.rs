use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{
        multipart::{Field, MultipartError},
        Multipart, Path, State,
    },
    response::{IntoResponse, Redirect},
    Extension, Json,
};
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    app::AppError,
    context::ContextTrait,
    easymde::{EditorConfig, UploadError, UploadSuccess},
    entity,
    property::File,
    render, Entity,
};

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
    let e = E::get(&id, ext).await.map_err(Into::into)?.ok_or_else(|| {
        AppError::new(
            "Not Found".to_string(),
            format!(
                "The {} with id {} does not exist",
                E::name().to_case(Case::Title),
                id
            ),
        )
    })?;
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
    debug!("creating entity {}", E::name());
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
    debug!("updating entity {}", E::name());
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
    debug!("deleting entity {}", E::name());
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
    #[error("Field must contain filename")]
    FilenameMissing,
    #[error("Failed to deserialize: {serde:#}: {query_string}")]
    Deserialize {
        serde: serde_qs::Error,
        query_string: String,
    },
}

async fn stream_field_to_file<'a>(
    mut field: Field<'a>,
    output_dir: &'a std::path::Path,
) -> Result<File, ParseFormError> {
    let id = Uuid::new_v4();
    let Some(filename) = field.file_name().filter(|name| !name.is_empty()) else {
        return Err(ParseFormError::FilenameMissing);
    };
    if filename.contains('/') {
        return Err(ParseFormError::FilenameSlash(filename.to_string()));
    }

    let folder_path = output_dir.join(id.to_string());
    tokio::fs::create_dir_all(&folder_path).await?;

    let file_path = folder_path.join(filename);

    // clone such that we don't keep a reference to filename for too long
    let filename = filename.to_string();

    let mut file = tokio::fs::File::create_new(file_path).await?;

    while let Some(v) = field.chunk().await? {
        file.write_all(&v).await?;
    }

    Ok(File::new_with_id(id, filename))
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
        let name = urlencoding::encode(name).to_string();
        match field.file_name() {
            Some(filename) if !filename.is_empty() => {
                let file = stream_field_to_file(field, files_dir).await?;
                let filename_escaped = urlencoding::encode(&file.name);
                let id = file.id;
                if !qs.is_empty() {
                    qs.push('&');
                }
                qs.push_str(&format!("{name}[name]={filename_escaped}&{name}[id]={id}"));
                // TODO: delete newly created files on error
            }
            None => {
                if !qs.is_empty() {
                    qs.push('&');
                }
                qs.push_str(&name);
                let bytes = field.bytes().await?;
                let value = urlencoding::encode_binary(&bytes);
                qs.push('=');
                qs.push_str(&value);
            }
            _ => {}
        };
    }
    serde_qs::Config::new(5, false)
        .deserialize_str(&qs)
        .map_err(|e| ParseFormError::Deserialize {
            serde: e,
            query_string: qs,
        })
}

#[derive(Clone, Debug)]
pub(crate) struct UploadDir(pub(crate) PathBuf);

pub(crate) async fn parse_mde_upload(
    config: Extension<EditorConfig>,
    path: Extension<UploadDir>,
    mut form: Multipart,
) -> Result<Json<UploadSuccess>, UploadError> {
    let upload_dir = path.0 .0;
    while let Some(field) = form.next_field().await? {
        let accepted = match field.content_type() {
            Some(content_type) => config.allowed_file_types.contains(&content_type.into()),
            None => false,
        };
        if !accepted {
            return Err(UploadError::TypeNotAllowed);
        }
        return match stream_field_to_file(field, &upload_dir).await {
            Ok(file) => Ok(Json::from(UploadSuccess::new(file.url()))),
            Err(err) => {
                error!("writing uploaded file failed: {err}");
                Err(UploadError::ImportError)
            }
        };
    }
    Err(UploadError::NoFileGiven)
}
