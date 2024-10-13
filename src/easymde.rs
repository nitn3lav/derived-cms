use std::{borrow::Cow, path::PathBuf};

use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use tracing::error;

#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// enable drag-and-drop upload functionality in the default markdown editor
    pub(crate) enable_uploads: bool,
    /// max upload size in bytes
    pub(crate) upload_max_size: u32,
    /// Allowed file types to upload. Default: image/png, image/jpeg
    pub(crate) allowed_file_types: Vec<Cow<'static, str>>,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            enable_uploads: true,
            upload_max_size: 1024 * 1024 * 2,
            allowed_file_types: vec!["image/png".into(), "image/jpeg".into()],
        }
    }
}

impl EditorConfig {
    /// Enable uploads directly in the editor.
    pub fn enable_uploads(mut self, enable: bool) -> Self {
        self.enable_uploads = enable;
        self
    }

    /// Set the max size for uploads.
    pub fn upload_max_size(mut self, max_size: u32) -> Self {
        self.upload_max_size = max_size;
        self
    }

    /// Add an allowed file type to the currently allowed file types.
    pub fn allow_file_type(mut self, file_type: impl Into<Cow<'static, str>>) -> Self {
        self.allowed_file_types.push(file_type.into());
        self
    }

    /// Reset the allowed file types to the given list.
    pub fn allowed_file_types(mut self, file_types: Vec<Cow<'static, str>>) -> Self {
        self.allowed_file_types = file_types;
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UploadedFileInfo {
    file_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UploadSuccess {
    data: UploadedFileInfo,
}

impl UploadSuccess {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            data: UploadedFileInfo {
                file_path: file_path.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "error")]
pub(crate) enum UploadError {
    NoFileGiven,
    TypeNotAllowed,
    FileTooLarge,
    ImportError,
}

impl From<MultipartError> for UploadError {
    fn from(err: MultipartError) -> Self {
        error!("multipart error while uploading from editor: {err}");
        match err.status() {
            StatusCode::PAYLOAD_TOO_LARGE => UploadError::FileTooLarge,
            _ => UploadError::ImportError,
        }
    }
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        let status_code = StatusCode::from_u16(match self {
            UploadError::NoFileGiven | UploadError::ImportError => 400,
            UploadError::TypeNotAllowed => 415,
            UploadError::FileTooLarge => 413,
        })
        .unwrap();
        let mut resp = Json::from(self).into_response();
        *resp.status_mut() = status_code;

        resp
    }
}
