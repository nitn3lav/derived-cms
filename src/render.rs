use std::collections::BTreeSet;

use axum::extract::{FromRef, State};
use convert_case::{Case, Casing};
use maud::{html, Markup, DOCTYPE};
use sqlx::Database;
use uuid::Uuid;

use crate::{property::EnumVariant, Entity};

pub trait ContextTrait<DB: Database>: Clone + Send + Sync {
    fn db(&self) -> &sqlx::Pool<DB>;
    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>>;
    fn ext(&self) -> &impl ContextExt<Self>;
}

#[derive(Debug)]
pub struct Context<DB: Database, T: ContextExt<Self>> {
    pub(crate) names_plural: BTreeSet<&'static str>,
    pub(crate) db: sqlx::Pool<DB>,
    pub(crate) ext: T,
}
impl<DB: Database, E: ContextExt<Self>> Clone for Context<DB, E> {
    fn clone(&self) -> Self {
        Self {
            names_plural: self.names_plural.clone(),
            db: self.db.clone(),
            ext: self.ext.clone(),
        }
    }
}
impl<DB: Database, E: ContextExt<Self>> ContextTrait<DB> for Context<DB, E> {
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

impl<DB: Database> FromRef<Context<DB, ()>> for () {
    fn from_ref(_input: &Context<DB, ()>) -> Self {}
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

pub fn add_entity<E: Entity<DB>, DB: Database>(value: Option<&E>) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        main {
            h1 {"Erstelle " (E::name().to_case(Case::Title))}
            form id=(form_id) class="cms-entity-form cms-add-form" method="post" {
                @for f in Entity::properties(value) {
                    div class="cms-prop-container" {
                        label for=(f.name) class="cms-prop-label" {(f.name)}
                        (f.value.render_input(f.name, f.name, &ctx))
                    }
                }
                button type="submit" {"Speichern"}
            }
        }
    }
}

pub async fn entity_list_page<E: Entity<DB>, DB: Database>(
    ctx: State<impl ContextTrait<DB>>,
) -> Markup {
    document(html! {
        (sidebar(ctx.names_plural(), E::name_plural()))
        (add_entity::<E, DB>(None))
    })
}

pub fn add_entity_page<E: Entity<DB>, DB: Database>(ctx: State<impl ContextTrait<DB>>) -> Markup {
    document(html! {
        (sidebar(ctx.names_plural(), E::name_plural()))
        (add_entity::<E, DB>(None))
    })
}

pub fn property_enum<'a>(
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
                div class=(class) {
                    @if let Some(ref data) = variant.content {
                        (data.value.render_input(variant.name, &variant.value.to_case(Case::Title), ctx))
                    }
                }
            }
        }
        script src="/js/enum.js" {}
    }
}
