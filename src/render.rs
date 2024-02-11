use std::{borrow::Borrow, cmp::Ordering, fmt::Display};

use axum::extract::State;
use convert_case::{Case, Casing};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use uuid::Uuid;

use crate::{
    context::ContextTrait, entity::EntityBase, input::InputInfo, property::EnumVariant, Entity,
};

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

pub fn entity_inputs<E: Entity<S>, S: ContextTrait>(
    i18n: &FluentLanguageLoader,
    value: Option<&E>,
) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        form id=(form_id) class="cms-entity-form cms-add-form" method="post" enctype="multipart/form-data" {
            (inputs(&ctx, i18n, EntityBase::inputs(value)))
            button class="cms-button" type="submit" {
                (fl!(i18n, "entity-inputs-submit"))
            }
            script src="/js/callOnMountRecursive.js" {}
            script {
                (PreEscaped(format!(r#"callOnMountRecursive(document.getElementById("{form_id}"));"#)))
            }
        }
    }
}

pub fn inputs<'a>(
    ctx: &FormRenderContext<'_>,
    i18n: &FluentLanguageLoader,
    inputs: impl IntoIterator<Item = InputInfo<'a>>,
) -> Markup {
    html! {
        @for f in inputs {
            div class="cms-prop-container" {
                label class="cms-prop-label" {(f.name_human)}
                (f.value.render_input(f.name, f.name_human, true, &ctx, i18n))
            }
        }
    }
}

pub fn entity_list_page<E: Entity<S>, S: ContextTrait>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entities: impl IntoIterator<Item = impl Borrow<E>>,
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
                    th {}
                }
                @for e in entities {
                    @let e = e.borrow();
                    @let name = E::name().to_case(Case::Kebab);
                    @let id = e.id().to_string();
                    @let id = urlencoding::encode(&id);
                    @let row_id = Uuid::new_v4();
                    @let dialog_id = Uuid::new_v4();
                    tr id=(row_id) {
                        @for c in e.column_values() {
                            td onclick=(format!(
                                "window.location = \"/{name}/{id}\"",
                            )) {
                                (c.render(i18n))
                            }
                        }
                        td
                            class="cms-list-delete-button"
                            onclick=(format!(r#"document.getElementById("{dialog_id}").showModal()"#))
                        {
                            "X"
                        }
                        (confirm_delete_modal(
                            i18n,
                            dialog_id,
                            &E::name().to_case(Case::Title),
                            format!(r#"
fetch("/{name}/{id}/delete", {{ method: "POST" }})
    .then(() => {{
        document.getElementById("{row_id}").remove();
        document.getElementById("{dialog_id}").remove();
    }})
                            "#).trim()
                        ))
                    }
                }
            }
        }
    })
}

pub fn confirm_delete_modal(
    i18n: &FluentLanguageLoader,
    dialog_id: impl Display,
    name: &str,
    on_submit: impl Display,
) -> Markup {
    html! {
        dialog id=(dialog_id) class="cms-confirm-delete-modal" {
            p {(fl!(i18n, "confirm-delete-modal", "title", name = name))}
            form method="dialog" {
                button {
                    (fl!(i18n, "confirm-delete-modal", "cancel"))
                }
                button onclick=(on_submit) {
                    (fl!(i18n, "confirm-delete-modal", "confirm"))
                }
            }
        }
    }
}

pub fn entity_page<E: Entity<S>, S: ContextTrait>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entity: Option<&E>,
) -> Markup {
    document(html! {
        (sidebar(i18n, ctx.names_plural(), E::name_plural()))
        main {
            h1 {(fl!(i18n, "edit-entity-title", name = E::name().to_case(Case::Title)))}
            (entity_inputs::<E, S>(i18n, entity))
        }
    })
}

pub fn add_entity_page<E: Entity<S>, S: ContextTrait>(
    ctx: State<impl ContextTrait>,
    i18n: &FluentLanguageLoader,
    entity: Option<&E>,
) -> Markup {
    document(html! {
        (sidebar(i18n, ctx.names_plural(), E::name_plural()))
        main {
            h1 {(fl!(i18n, "create-entity-title", name = E::name().to_case(Case::Title)))}
            (entity_inputs::<E, S>(i18n, entity))
        }
    })
}

pub fn input_enum(
    ctx: &FormRenderContext<'_>,
    i18n: &FluentLanguageLoader,
    variants: &[EnumVariant<'_>],
    selected: usize,
    required: bool,
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
                    checked[i == selected]
                    onchange="cmsEnumInputOnchange(this)" {}
                label for=(id) {(variant.value.to_case(Case::Title))}
            }
        }
        div class="cms-enum-data" id=(id_data) {
            @for (i, variant) in variants.iter().enumerate() {
                @let class = match i.cmp(&selected) {
                    Ordering::Less => "cms-enum-container cms-enum-hidden cms-enum-hidden-left",
                    Ordering::Greater => "cms-enum-container cms-enum-hidden cms-enum-hidden-right",
                    Ordering::Equal => "cms-enum-container",
                };
                fieldset class=(class) disabled[i != selected] {
                    @if let Some(ref data) = variant.content {
                        (data.value.render_input(data.name, &variant.value.to_case(Case::Title), required, ctx, i18n))
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
                @for line in description.split('\n') {
                    (line)
                    br;
                }
            }
            a href="javascript:history.back()" {"Go Back"}
        }
    })
}
