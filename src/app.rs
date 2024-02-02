use std::{collections::BTreeSet, path::PathBuf, sync::Arc};

use axum::{
    extract::Request,
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use include_dir::{include_dir, Dir, DirEntry};
use rust_embed::RustEmbed;
use tower_http::services::ServeDir;
use unic_langid::LanguageIdentifier;

use crate::{
    context::{Context, ContextExt},
    entity::Entity,
    DB,
};

static STATIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

/// build an [`axum::Router`] with all routes required for API and admin interface
#[derive(Clone, Debug)]
pub struct App<S, E>
where
    S: ContextExt<Context<S>>,
{
    router: Router<Context<S>>,
    names_plural: BTreeSet<&'static str>,
    state_ext: E,
}

impl<S> Default for App<S, ()>
where
    S: ContextExt<Context<S>> + 'static,
{
    fn default() -> Self {
        Self {
            router: Default::default(),
            names_plural: Default::default(),
            state_ext: Default::default(),
        }
    }
}

impl<S> App<S, ()>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S, SE> App<S, SE>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn entity<E: Entity<Context<S>> + Send + Sync>(mut self) -> Self {
        self.names_plural.insert(E::name_plural());
        self.router = self.router.merge(E::routes());
        self
    }
}

impl<S, E> App<S, E>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn with_state(self, data: S) -> App<S, S> {
        App {
            router: self.router,
            names_plural: self.names_plural,
            state_ext: data,
        }
    }
}

impl<S> App<S, S>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn build(self, uploads_dir: impl Into<PathBuf>, db: sqlx::Pool<DB>) -> Router {
        let uploads_dir = uploads_dir.into();
        self.router
            .nest_service("/uploads", ServeDir::new(&uploads_dir))
            .with_state(Context {
                names_plural: self.names_plural,
                db,
                uploads_dir,
                ext: self.state_ext,
            })
            .layer(middleware::from_fn(|mut req: Request, next: Next| {
                // add extension `()` to prevent HTTP 500 response when using default/derived impl of `EntityHooks`.
                req.extensions_mut().insert(());
                next.run(req)
            }))
            .layer(middleware::from_fn(localize))
            .merge(include_static_files(&STATIC_ASSETS))
    }
}

async fn localize(mut req: Request, next: Next) -> Response {
    let langs = req
        .headers()
        .get(axum::http::header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .map(accept_language::parse)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|lang| lang.parse::<LanguageIdentifier>().ok())
        .collect::<Vec<_>>();
    let language_loader: FluentLanguageLoader = fluent_language_loader!();
    i18n_embed::select(&language_loader, &Localizations, &langs).unwrap();
    req.extensions_mut().insert(Arc::new(language_loader));
    next.run(req).await
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
