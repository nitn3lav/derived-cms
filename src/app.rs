use std::{convert::Infallible, path::PathBuf, sync::Arc};

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use derive_more::Debug;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    AssetsMultiplexor, I18nAssets,
};
use include_dir::{include_dir, Dir, DirEntry};
use rust_embed::RustEmbed;
use tower_http::services::ServeDir;
use tracing::error;
use unic_langid::LanguageIdentifier;

use crate::{
    context::{Context, ContextExt},
    easymde::EditorConfig,
    endpoints::{
        entity_routes,
        ui::{parse_mde_upload, UploadDir},
    },
    entity::Entity,
    render,
};

static STATIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

/// build an [`axum::Router`] with all routes required for API and admin interface
#[derive(Debug)]
pub struct App<S, E>
where
    S: ContextExt<Context<S>>,
{
    router: Router<Context<S>>,
    names_plural: Vec<&'static str>,
    editor_config: Option<EditorConfig>,
    state_ext: E,
    #[debug(skip)]
    localizations: Vec<Box<dyn I18nAssets + Send + Sync + 'static>>,
}

impl<S> Default for App<S, ()>
where
    S: ContextExt<Context<S>> + 'static,
{
    fn default() -> Self {
        Self {
            router: Default::default(),
            names_plural: Default::default(),
            editor_config: None,
            state_ext: Default::default(),
            localizations: Vec::new(),
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
        self.names_plural.push(E::name_plural());
        self.router = self.router.merge(entity_routes::<E, Context<S>>());
        self
    }
}

impl<S, SE> App<S, SE>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn with_mdeditor(mut self, config: EditorConfig) -> Self {
        self.editor_config = Some(config);
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
            editor_config: self.editor_config,
            state_ext: data,
            localizations: self.localizations,
        }
    }
}

impl<S, E> App<S, E>
where
    S: ContextExt<Context<S>> + 'static,
{
    //! Include the given assets when loading the localized messages
    //! upon a request.
    //! They can then be accessed in any render functions called by
    //! this library (e.g. [`Column::render`](crate::column::Column::render)
    //! or [`Input::render_input`](crate::input::Input::render_input)).
    //! Note: The domain in these functions will still be `"derived_cms"`,
    //! so make sure to add the localized messages to the correct domain
    //! file in the assets.
    pub fn include_localizations(
        self,
        assets: impl I18nAssets + Send + Sync + 'static,
    ) -> App<S, E> {
        let mut localizations = self.localizations;
        localizations.push(Box::new(assets));
        App {
            localizations,
            ..self
        }
    }
}

impl<S> App<S, S>
where
    S: ContextExt<Context<S>> + 'static,
{
    pub fn build(self, uploads_dir: impl Into<PathBuf>) -> Router {
        let uploads_dir = uploads_dir.into();

        let mut localizations = self.localizations;
        localizations.push(Box::new(Localizations));
        let localizations = Arc::new(AssetsMultiplexor::new(localizations));

        let mut router = self
            .router
            .nest_service("/uploads", ServeDir::new(&uploads_dir))
            .with_state(Context {
                names_plural: self.names_plural,
                editor_config: self.editor_config.clone(),
                uploads_dir: uploads_dir.clone(),
                ext: self.state_ext,
            })
            .layer(middleware::from_fn(|mut req: Request, next: Next| {
                // add extension `()` to prevent HTTP 500 response when using default/derived impl of `EntityHooks`.
                req.extensions_mut().insert(());
                next.run(req)
            }))
            .layer(middleware::from_fn_with_state(localizations, localize))
            .merge(include_static_files(&STATIC_ASSETS));
        if let Some(editor_config) = self.editor_config.filter(|config| config.enable_uploads) {
            router = router.route(
                "/upload",
                post(parse_mde_upload)
                    .layer::<_, Infallible>(DefaultBodyLimit::max(
                        editor_config.upload_max_size as usize,
                    ))
                    .layer::<_, Infallible>(Extension(editor_config))
                    .layer(Extension(UploadDir(uploads_dir))),
            );
        }

        router
    }
}

async fn localize(
    State(localizations): State<Arc<AssetsMultiplexor>>,
    mut req: Request,
    next: Next,
) -> Response {
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
    i18n_embed::select(&language_loader, &*localizations, &langs).unwrap();
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

pub struct AppError {
    pub title: String,
    pub description: String,
}

impl From<()> for AppError {
    fn from(_value: ()) -> Self {
        Self {
            title: "Infallible".to_string(),
            description: "Infallible".to_string(),
        }
    }
}

impl AppError {
    pub fn new(title: String, description: String) -> Self {
        Self { title, description }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("{}: {}", self.title, self.description);
        (
            StatusCode::BAD_REQUEST,
            render::error_page(&self.title, &self.description),
        )
            .into_response()
    }
}
