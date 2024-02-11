use axum::{
    routing::{delete, get, post},
    Router,
};
use convert_case::{Case, Casing};

use crate::{context::ContextTrait, Entity};

pub mod api;
pub mod ui;

/// returns a [Router] with all generated HTTP endponts
pub fn entity_routes<E: Entity<S>, S: ContextTrait>() -> Router<S> {
    let name = E::name().to_case(Case::Kebab);
    let name = urlencoding::encode(&name);
    let name_pl = E::name_plural().to_case(Case::Kebab);
    let name_pl = urlencoding::encode(&name_pl);

    Router::new()
        // API
        .route(
            &format!("/api/v1/{name_pl}"),
            get(api::get_entities::<E, S>),
        )
        .route(&format!("/api/v1/{name}/:id"), get(api::get_entity::<E, S>))
        .route(
            &format!("/api/v1/{name_pl}"),
            post(api::post_entities::<E, S>),
        )
        .route(
            &format!("/api/v1/{name}/:id"),
            post(api::post_entity::<E, S>),
        )
        .route(
            &format!("/api/v1/{name}/:id"),
            delete(api::delete_entity::<E, S>),
        )
        // UI
        .route(&format!("/{name_pl}"), get(ui::get_entities::<E, S>))
        .route(&format!("/{name}/:id"), get(ui::get_entity::<E, S>))
        .route(&format!("/{name}/:id"), post(ui::post_entity::<E, S>))
        .route(&format!("/{name_pl}/add"), get(ui::get_add_entity::<E, S>))
        .route(
            &format!("/{name_pl}/add"),
            post(ui::post_add_entity::<E, S>),
        )
        .route(
            &format!("/{name}/:id/delete"),
            post(ui::delete_entity::<E, S>),
        )
}
