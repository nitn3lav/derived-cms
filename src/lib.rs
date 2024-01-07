//! Generate a CMS, complete with admin interface, headless API and database interface from Rust
//! type definitions. Works in cunjunction with [serde] and [sqlx] and uses [axum] as a web
//! server.
//!
//! Example
//!
//! ```no_run
//! use chrono::{DateTime, Utc};
//! use derived_cms::{App, Entity, Property, property::{Markdown, Text}};
//! use serde::{Deserialize, Serialize};
//! use sqlx::prelude::*:
//!
//! #[derive(Debug, Deserialize, Serialize, Entity)]
//! #[serde(rename_all = "snake_case")]
//! struct Post {
//!     title: Text,
//!     date: DateTime<Utc>,
//!     #[sqlx(json)]
//!     content: Vec<Block>,
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
//!     let app = App::new().entity::<Post>().build();
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
