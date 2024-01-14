# derived-cms

Generate a CMS, complete with admin interface, headless API and database interface from Rust
type definitions. Works in cunjunction with [serde](https://docs.rs/serde/latest/serde/) and
[ormlite](https://lib.rs/crates/ormlite) and uses [axum](https://docs.rs/axum/latest/axum/)
as a web server.

Example

```rust
use chrono::{DateTime, Utc};
use derived_cms::{App, Entity, Input, property::{Markdown, Text, Json}};
use ormlite::{Model, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Entity, Model, TS)]
#[ts(export)]
struct Post {
    #[cms(skip_input)]
    #[ormlite(primary_key)]
    #[serde(default = "uuid::Uuid::new_v4")]
    id: Uuid,
    title: Text,
    date: DateTime<Utc>,
    #[cms(skip_column)]
    #[serde(default)]
    content: Json<Vec<Block>>,
    draft: bool,
}

#[derive(Debug, Deserialize, Serialize, Input, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Block {
    Separator,
    Text(Markdown),
}

#[tokio::main]
async fn main() {
    let db = sqlx::Pool::<Sqlite>::connect("sqlite://.tmp/db.sqlite")
        .await
        .unwrap();
    let app = App::new().entity::<Post>().build(db);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Hooks

You can add hooks to be run before an entity is created or updated

```rust
impl EntityHooks for Post {
    // can be used to pass state from a custom middleware
    type RequestExt<S: ContextTrait> = ();

    async fn on_create(self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
        // do some stuff
        Ok(self)
    }

    async fn on_update(old: Self, new: Self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
        // do some stuff
        Ok(new)
    }

    async fn on_delete(self, ext: Self::RequestExt<impl ContextTrait>) -> Result<Self, Infallible> {
        // do some stuff
        Ok(self)
    }
}
```

## REST API

A REST API is automatically generated for all `Entities`.

List of generated endpoints, with `name` and `name-plural`
converted to kebab-case:

- `GET /api/v1/:name-plural`:
  - allows filtering by exact value in the query string, e. g. `?slug=asdf`. This currently
    only works for fields whose SQL representation is a string.
  - returns an array of entities, serialized using [serde_json](https://docs.rs/serde-json/latest/serde_json).
- `GET /api/v1/:name/:id`
  - get an Entity by it's id.
  - returns the requested of Entity, serialized using [serde_json](https://docs.rs/serde-json/latest/serde_json).
- `POST /api/v1/:name-plural`
  - create a new Entity from the request body JSON.
  - returns the newly created Entity as JSON.
- `POST /api/v1/:name/:id`
  - replaces the Entity with the specified id with the
    request body JSON.
  - returns the updated Entity as JSON.
- `DELETE /api/v1/:name/:id`
  - deletes the Entity with the specified id
  - returns the deleted Entity as JSON.
