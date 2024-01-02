use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use convert_case::{Case, Casing};
use include_dir::{include_dir, Dir, DirEntry};
use maud::{html, Markup, DOCTYPE};
use uuid::Uuid;

use crate::{entity::Entity, property::FormRenderContext};

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
            get(|| async { document(render_add_entity::<E>(None)) }),
        );
        self
    }

    pub fn build(self) -> Router {
        self.router.nest("/", include_static_files(&STATIC_ASSETS))
    }
}

fn render_add_entity<E: Entity>(value: Option<&E>) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        main {
            h1 {"Erstelle neues " (E::name().to_case(Case::Title))}
            form id=(form_id) class="cms-entity-form cms-add-form" method="post" {
                @for f in Entity::properties(value) {
                    div {
                        label for=(f.name) {(f.name)}
                        (f.value.render_input(f.name, f.name, &ctx))
                    }
                }
                button type="submit" {"Speichern"}
            }
        }
    }
}

fn document(body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                link rel="stylesheet" href="" {}
                meta charset="utf-8" {}
                link rel="icon" href="/favicon.png" {}
                link rel="stylesheet" type="text/css" href="/css/main.css" {}
                meta name="viewport" content="width=device-width, initial-scale=1" {}
            }
            body {
                (body)
            }
        }
    }
}

fn include_static_files(dir: &'static Dir<'_>) -> Router {
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
