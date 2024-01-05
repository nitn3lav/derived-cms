use axum::{extract::State, response::IntoResponse};

use crate::{render, Entity};

pub async fn post_add_entity<E: Entity, S: render::ContextTrait>(
    ctx: State<S>,
) -> impl IntoResponse {
    render::add_entity_page::<E>(ctx)
}
