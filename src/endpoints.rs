use axum::{extract::State, response::IntoResponse};
use sqlx::Database;

use crate::{render, Entity};

pub async fn get_add_entity<E: Entity<DB>, DB: Database, S: render::ContextTrait<DB>>(
    ctx: State<S>,
) -> impl IntoResponse {
    render::add_entity_page::<E, DB>(ctx)
}
