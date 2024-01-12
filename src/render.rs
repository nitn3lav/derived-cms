use axum::extract::State;
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use maud::{html, Markup, DOCTYPE};
use uuid::Uuid;

use crate::{context::ContextTrait, property::EnumVariant, Entity};

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

pub fn sidebar(
    _i18n: &FluentLanguageLoader,
    names: impl IntoIterator<Item = impl AsRef<str>>,
    active: &str,
) -> Markup {
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

pub fn entity_inputs<E: Entity>(i18n: &FluentLanguageLoader, value: Option<&E>) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        form id=(form_id) class="cms-entity-form cms-add-form" method="post" {
            @for f in Entity::inputs(value) {
                div class="cms-prop-container" {
                    label class="cms-prop-label" {(f.name)}
                    (f.value.render_input(f.name, f.name, &ctx, i18n))
                }
            }
            button class="cms-button" type="submit" {
                (fl!(i18n, "entity-inputs-submit"))
            }
        }
    }
}

pub fn entity_list_page<E: Entity>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entities: &[E],
) -> Markup {
    document(html! {
        (sidebar(&i18n, ctx.names_plural(), E::name_plural()))
        main {
            header class="cms-header" {
                h1 {(E::name_plural().to_case(Case::Title))}
                a href=(format!("/{}/add", (E::name_plural().to_case(Case::Kebab)))) class="cms-button" {
                    (fl!(i18n, "enitity-list-add"))
                }
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

pub fn entity_page<E: Entity>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entity: Option<&E>,
) -> Markup {
    document(html! {
        (sidebar(i18n, ctx.names_plural(), E::name_plural()))
        main {
            h1 {(fl!(i18n, "edit-entity-title", name = E::name().to_case(Case::Title)))}
            (entity_inputs::<E>(i18n, entity))
        }
    })
}

pub fn add_entity_page<E: Entity>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entity: Option<&E>,
) -> Markup {
    document(html! {
        (sidebar(i18n, ctx.names_plural(), E::name_plural()))
        main {
            h1 {(fl!(i18n, "create-entity-title", name = E::name().to_case(Case::Title)))}
            (entity_inputs::<E>(i18n, entity))
        }
    })
}

pub fn input_enum<'a>(
    ctx: &FormRenderContext<'a>,
    i18n: &FluentLanguageLoader,
    variants: &[EnumVariant<'a>],
    selected: usize,
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
                        (data.value.render_input(data.name, &variant.value.to_case(Case::Title), ctx, i18n))
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
