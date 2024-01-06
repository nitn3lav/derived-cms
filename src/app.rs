use std::collections::BTreeSet;

use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    routing::get,
    Router,
};

use include_dir::{include_dir, Dir, DirEntry};
use sqlx::Database;

use crate::{entity::Entity, render};

static STATIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// build an [`axum::Router`] with all routes required for API and admin interface
#[derive(Clone, Debug)]
pub struct App<DB, S, E>
where
    DB: Database,
    S: render::ContextExt<render::Context<DB, S>>,
{
    router: Router<render::Context<DB, S>>,
    names_plural: BTreeSet<&'static str>,
    state_ext: E,
}

impl<DB, S> App<DB, S, ()>
where
    DB: Database,
    S: render::ContextExt<render::Context<DB, S>> + 'static,
{
    pub fn new() -> Self {
        Self {
            router: Router::<render::Context<DB, S>>::new(),
            names_plural: Default::default(),
            state_ext: Default::default(),
        }
    }
}

impl<DB, S, SE> App<DB, S, SE>
where
    DB: Database,
    S: render::ContextExt<render::Context<DB, S>> + 'static,
{
    pub fn entity<E: Entity<DB> + Send + Sync>(mut self) -> Self {
        self.names_plural.insert(E::name_plural());
        self.router = self.router.merge(E::routes::<render::Context<DB, S>>());
        self
    }
}

impl<DB, S, E> App<DB, S, E>
where
    DB: Database,
    S: render::ContextExt<render::Context<DB, S>> + 'static,
{
    pub fn with_state(self, data: S) -> App<DB, S, S> {
        App {
            router: self.router,
            names_plural: self.names_plural,
            state_ext: data,
        }
    }
}

impl<DB, S> App<DB, S, S>
where
    DB: Database,
    S: render::ContextExt<render::Context<DB, S>> + 'static,
{
    pub fn build(self, db: sqlx::Pool<DB>) -> Router {
        self.router
            .with_state(render::Context {
                names_plural: self.names_plural,
                db,
                ext: self.state_ext,
            })
            .merge(include_static_files(&STATIC_ASSETS))
    }
}

pub fn include_static_files<S: Clone + Send + Sync + 'static>(dir: &'static Dir<'_>) -> Router<S> {
    let mut app = Router::<S>::new();
    for v in dir.entries() {
        match v {
            DirEntry::Dir(d) => app = app.merge(include_static_files(d)),
            DirEntry::File(f) => {
                if let Some(path) = f.path().to_str() {
                    let mime = mime_guess::from_path(path)
                        .first_or_octet_stream()
                        .to_string();
                    let headers = HeaderMap::from_iter([(
                        CONTENT_TYPE,
                        HeaderValue::from_str(&mime).unwrap(),
                    )]);
                    app = app.route(
                        &format!("/{path}"),
                        get(move || async move { (headers, f.contents()) }),
                    )
                }
            }
        }
    }
    app
}
