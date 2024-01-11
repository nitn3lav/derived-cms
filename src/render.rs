use std::collections::BTreeSet;

use axum::extract::{FromRef, State};
use convert_case::{Case, Casing};
use maud::{html, Markup, DOCTYPE};
use uuid::Uuid;

use crate::{property::EnumVariant, Entity, DB};

pub trait ContextTrait: Clone + Send + Sync {
    fn db(&self) -> &sqlx::Pool<DB>;
    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>>;
    fn ext(&self) -> &impl ContextExt<Self>;
}

#[derive(Debug)]
pub struct Context<T: ContextExt<Self>> {
    pub(crate) names_plural: BTreeSet<&'static str>,
    pub(crate) db: sqlx::Pool<DB>,
    pub(crate) ext: T,
}
impl<E: ContextExt<Self>> Clone for Context<E> {
    fn clone(&self) -> Self {
        Self {
            names_plural: self.names_plural.clone(),
            db: self.db.clone(),
            ext: self.ext.clone(),
        }
    }
}
impl<E: ContextExt<Self>> ContextTrait for Context<E> {
    fn db(&self) -> &sqlx::Pool<DB> {
        &self.db
    }
    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>> {
        self.names_plural.iter()
    }
    fn ext(&self) -> &impl ContextExt<Self> {
        &self.ext
    }
}

impl FromRef<Context<()>> for () {
    fn from_ref(_input: &Context<()>) -> Self {}
}

pub trait ContextExt<Ctx>: FromRef<Ctx> + Clone + Send + Sync {}

impl<Ctx, T: Send + Sync + 'static> ContextExt<Ctx> for T where T: FromRef<Ctx> + Clone {}

#[non_exhaustive]
pub struct FormRenderContext<'a> {
    /// unique id of the HTML form element
    pub form_id: &'a str,
}

pub fn document(body: Markup) -> Markup {
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

pub fn sidebar(names: impl IntoIterator<Item = impl AsRef<str>>, active: &str) -> Markup {
    html! {
        nav class="cms-sidebar" {
            @for name in names {
                @let name = name.as_ref();
                a href=(&format!("/{}", name.to_case(Case::Kebab))) class=[(name == active).then_some("active")] {
                    (name.to_case(Case::Title))
                }
            }
        }
    }
}

pub fn add_entity<E: Entity>(value: Option<&E>) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        main {
            h1 {"Erstelle " (E::name().to_case(Case::Title))}
            form id=(form_id) class="cms-entity-form cms-add-form" method="post" {
                @for f in Entity::inputs(value) {
                    div class="cms-prop-container" {
                        label class="cms-prop-label" {(f.name)}
                        (f.value.render_input(f.name, f.name, &ctx))
                    }
                }
                button type="submit" {"Speichern"}
            }
        }
    }
}

pub fn entity_list_page<E: Entity>(ctx: State<impl ContextTrait>, entities: &[E]) -> Markup {
    document(html! {
        (sidebar(ctx.names_plural(), E::name_plural()))
        main {
            header class="cms-entity-list-header" {
                h1 {(E::name_plural().to_case(Case::Title))}
                a href=(format!("/{}/add", (E::name_plural().to_case(Case::Kebab)))) class="cms-header-button" {"Create new"}
            }
            table class="cms-entity-list" {
                tr {
                    @for c in E::column_names() {
                        th {(c)}
                    }
                }
                @for e in entities {
                    tr {
                        @for c in e.column_values() {
                            td onclick=(format!(
                                "window.location = \"/{}/{}\"",
                                E::name().to_case(Case::Kebab),
                                urlencoding::encode(&e.id().to_string()))
                            ) {
                                (c.render())
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn add_entity_page<E: Entity>(ctx: State<impl ContextTrait>) -> Markup {
    document(html! {
        (sidebar(ctx.names_plural(), E::name_plural()))
        (add_entity::<E>(None))
    })
}

pub fn input_enum<'a>(
    variants: &[EnumVariant<'a>],
    selected: usize,
    ctx: &FormRenderContext<'a>,
) -> Markup {
    let id_type = Uuid::new_v4();
    let id_data = Uuid::new_v4();
    html! {
        div class="cms-enum-type" id=(id_type) {
            @for (i, variant) in variants.iter().enumerate() {
                @let id = &format!("{}_radio-button_{}", variant.name, variant.value);
                input
                    type="radio"
                    name=(variant.name)
                    value=(variant.value)
                    id=(id)
                    onchange="cmsEnumInputOnchange(this)"
                    checked[i == selected] {}
                label for=(id) {(variant.value.to_case(Case::Title))}
            }
        }
        div class="cms-enum-data" id=(id_data) {
            @for (i, variant) in variants.iter().enumerate() {
                @let class = if i < selected {
                    "cms-enum-container cms-enum-hidden cms-enum-hidden-left"
                } else if i > selected {
                    "cms-enum-container cms-enum-hidden cms-enum-hidden-right"
                } else {
                    "cms-enum-container"
                };
                fieldset class=(class) disabled[i != selected] {
                    @if let Some(ref data) = variant.content {
                        (data.value.render_input(data.name, &variant.value.to_case(Case::Title), ctx))
                    }
                }
            }
        }
        script src="/js/enum.js" {}
    }
}

pub fn error_page(title: &str, description: &str) -> Markup {
    document(html! {
        main {
            h1 {(title)}
            p {
                @for line in description.split("\n") {
                    (line)
                    br;
                }
            }
            a href="javascript:history.back()" {"Go Back"}
        }
    })
}
