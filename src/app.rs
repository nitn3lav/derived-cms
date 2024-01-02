use std::{collections::BTreeSet, marker::PhantomData};

use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    routing::get,
    Router,
};

use include_dir::{include_dir, Dir, DirEntry};
use maud::html;

use crate::{entity::Entity, render};

static STATIC_ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/static");

/// builds an [`axum::Router`] with all routes required by the admin interface
#[derive(Debug, Default)]
pub struct App<T = ()>
where
    T: 'static,
{
    router: Router,
    names_plural: BTreeSet<&'static str>,
    phantom: PhantomData<&'static T>,
}
impl<T> Clone for App<T> {
    fn clone(&self) -> Self {
        Self {
            router: self.router.clone(),
            names_plural: self.names_plural.clone(),
            phantom: PhantomData,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Default::default()
    }
}
impl<T> App<T> {
    pub fn entity<E: Entity + Send + Sync>(mut self) -> App<(T, E)> {
        self.names_plural.insert(E::name_plural());
        App {
            router: self.router,
            names_plural: self.names_plural,
            phantom: PhantomData,
        }
    }

    fn build_end(self) -> Router {
        self.router.nest("/", include_static_files(&STATIC_ASSETS))
    }
    fn build_entity<E: Entity>(self) -> Router {
        self.router.route(
            &format!("/{}/add", E::name_plural()),
            get(move || async move {
                render::document(html! {
                    (render::sidebar(self.names_plural, E::name_plural()))
                    (render::add_entity::<E>(None))
                })
            }),
        )
    }
}
impl<T: BuildApp> App<T> {
    pub fn build(self) -> Router {
        T::build(self)
    }
}

pub trait BuildApp {
    fn build<O>(app: App<O>) -> Router;
}
impl BuildApp for () {
    fn build<O>(app: App<O>) -> Router {
        app.build_end()
    }
}
impl<E: Entity> BuildApp for E {
    fn build<O>(app: App<O>) -> Router {
        app.build_entity::<E>()
    }
}
impl<T: BuildApp, E: Entity> BuildApp for (T, E) {
    fn build<O>(app: App<O>) -> Router {
        T::build(app.clone()).nest("/", E::build(app))
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
