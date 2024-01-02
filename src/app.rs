use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    routing::get,
    Router,
};

use include_dir::{include_dir, Dir, DirEntry};

use crate::{entity::Entity, render};

static STATIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// builds an [`axum::Router`] with all routes required by the admin interface
#[derive(Debug, Default)]
pub struct App {
    router: Router,
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn entity<E: Entity + Send + Sync>(mut self) -> Self {
        self.router = self.router.route(
            &format!("/{}/add", E::name_plural()),
            get(|| async { render::document(render::add_entity::<E>(None)) }),
        );
        self
    }

    pub fn build(self) -> Router {
        self.router.nest("/", include_static_files(&STATIC_ASSETS))
    }
}

pub fn include_static_files(dir: &'static Dir<'_>) -> Router {
    let mut app = Router::new();
    for v in dir.entries() {
        match v {
            DirEntry::Dir(d) => app = app.nest("/", include_static_files(d)),
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
