//! Generate a CMS, complete with admin interface, headless API and database interface from Rust
//! type definitions. Works in cunjunction with [serde] and [ormlite] and uses [axum] as a web
//! server.
//!
//! Example
//!
//! ```no_run
//! use chrono::{DateTime, Utc};
//! use derived_cms::{App, Entity, Property, property::{Markdown, Text}};
//! use ormlite::{Model, sqlite::Sqlite, types::Json};
//! use serde::{Deserialize, Serialize};
//! use uuid::Uuid;
//!
//! #[derive(Debug, Deserialize, Serialize, Model, Entity)]
//! struct Post {
//!     #[cms(skip_input)]
//!     #[ormlite(primary_key)]
//!     #[serde(default = "uuid::Uuid::new_v4")]
//!     id: Uuid,
//!     title: Text,
//!     date: DateTime<Utc>,
//!     #[serde(default)]
//!     content: Json<Vec<Block>>,
//!     draft: bool,
//! }
//!
//! #[derive(Debug, Deserialize, Serialize, Property)]
//! #[serde(rename_all = "snake_case", tag = "type", content = "data")]
//! pub enum Block {
//!     Separator,
//!     Text(Markdown),
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let db = sqlx::Pool::<Sqlite>::connect("sqlite://.tmp/db.sqlite?mode=rwc")
//!         .await
//!         .unwrap();
//!     let app = App::new().entity::<Post>().build(db);
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```

pub use app::App;
pub use entity::Entity;
pub use property::Property;

pub mod app;
mod endpoints;
pub mod entity;
pub mod property;
pub mod render;

#[doc(hidden)]
pub mod derive {
    pub use generic_array;
    pub use maud;
    pub use ormlite;
}
