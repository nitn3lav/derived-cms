//! Generate a CMS, complete with admin interface, headless API and database interface from Rust
//! type definitions. Works in cunjunction with [serde] and [ormlite] and uses [axum] as a web
//! server.
//!
//! Example
//!
//! ```rust,no_run
//! use chrono::{DateTime, Utc};
//! use derived_cms::{App, Entity, Input, property::{Markdown, Text}};
//! use ormlite::{Model, sqlite::Sqlite, types::Json};
//! use serde::{Deserialize, Serialize};
//! use uuid::Uuid;
//!
//! #[derive(Debug, Deserialize, Serialize, Model, Entity)]
//! struct Post {
//!     #[cms(id, skip_input)]
//!     #[ormlite(primary_key)]
//!     #[serde(default = "uuid::Uuid::new_v4")]
//!     id: Uuid,
//!     title: Text,
//!     date: DateTime<Utc>,
//!     #[cms(skip_column)]
//!     #[serde(default)]
//!     content: Json<Vec<Block>>,
//!     draft: bool,
//! }
//!
//! #[derive(Debug, Deserialize, Serialize, Input)]
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
//!
//! ## Hooks
//!
//! You can add hooks to be run before an entity is created or updated
//!
//! ```rust
//! use std::convert::Infallible;
//!
//! use chrono::{DateTime, Utc};
//! use derived_cms::{App, Entity, EntityHooks, Input, context::ContextTrait, property::{Markdown, Text}};
//! use ormlite::{Model, sqlite::Sqlite, types::Json};
//! use serde::{Deserialize, Serialize};
//! use uuid::Uuid;
//!
//! #[derive(Debug, Deserialize, Serialize, Model, Entity)]
//! #[cms(hooks)]
//! struct Post {
//!     #[cms(id, skip_input)]
//!     #[ormlite(primary_key)]
//!     #[serde(default = "uuid::Uuid::new_v4")]
//!     id: Uuid,
//!     title: Text,
//!     date: DateTime<Utc>,
//!     draft: bool,
//! }
//!
//! impl EntityHooks for Post {
//!     // can be used to pass state from a custom middleware
//!     type RequestExt<S: ContextTrait> = ();
//!
//!     async fn on_create(self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
//!         // do some stuff
//!         Ok(self)
//!     }
//!
//!     async fn on_update(self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
//!         // do some stuff
//!         Ok(self)
//!     }
//!
//!     async fn on_delete(self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
//!         // do some stuff
//!         Ok(self)
//!     }
//! }
//! ```

pub use app::App;
pub use column::Column;
pub use entity::{Entity, EntityHooks};
pub use input::Input;

pub mod app;
pub mod column;
pub mod context;
mod endpoints;
pub mod entity;
pub mod input;
pub mod property;
pub mod render;

#[doc(hidden)]
pub mod derive {
    pub use generic_array;
    pub use i18n_embed;
    pub use maud;
    pub use ormlite;
}

#[cfg(feature = "sqlite")]
pub type DB = sqlx::Sqlite;
#[cfg(feature = "postgres")]
pub type DB = sqlx::Postgres;
