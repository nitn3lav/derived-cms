# derived-cms

Generate a CMS, complete with admin interface, headless API and database interface from Rust
type definitions. Works in cunjunction with [serde](https://docs.rs/serde/latest/serde/) and [sea_orm](https://www.sea-ql.org/SeaORM/)
and uses [axum](https://docs.rs/axum/latest/axum/) as a web server.

Example

```rust
use chrono::{DateTime, Utc};
use derived_cms::{App, Entity, Property, property::{Markdown, Text}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Entity)]
#[serde(rename_all = "snake_case")]
struct Post {
    title: Text,
    date: DateTime<Utc>,
    content: Vec<Block>,
    draft: bool,
}

#[derive(Debug, Deserialize, Serialize, Property)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Block {
    Separator,
    Text(Markdown),
}

#[tokio::main]
async fn main() {
    let app = App::new().entity::<Post>().build();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```
