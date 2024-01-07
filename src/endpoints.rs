use axum::{
    extract::{RawForm, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use convert_case::{Case, Casing};
use sqlx::Database;
use tracing::{debug, error};

use crate::{render, Entity};

pub async fn get_add_entity<E: Entity<DB>, DB: Database, S: render::ContextTrait<DB>>(
    ctx: State<S>,
) -> impl IntoResponse {
    render::add_entity_page::<E, DB>(ctx)
}

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

pub async fn post_add_entity<E: Entity<DB>, DB: Database, S: render::ContextTrait<DB>>(
    ctx: State<S>,
    RawForm(form): RawForm,
) -> Result<impl IntoResponse, AppError> {
    let db = ctx.db();
    let x = serde_qs::Config::new(5, false)
        .deserialize_bytes::<E>(&form)
        .map_err(|e| {
            AppError::new(
                format!("Failed to create new {}", E::name().to_case(Case::Title)),
                format!("Failed to parse form: {e}"),
            )
        })?;
    debug!(
        "Creating new {}: {}",
        E::name().to_case(Case::Title),
        serde_json::to_string(&x).unwrap()
    );
    let x: E = x.insert(db).await.map_err(|e| {
        AppError::new(
            format!("Failed to create new {}", E::name().to_case(Case::Title)),
            format!("Database error: {e}"),
        )
    })?;
    debug!(
        "Created new {}: {}",
        E::name().to_case(Case::Title),
        serde_json::to_string(&x).unwrap()
    );

    let uri = &format!("/{}/{}", E::name().to_case(Case::Kebab), "id");
    Ok(Redirect::to(uri))
}
