use convert_case::{Case, Casing};
use maud::{html, Markup, DOCTYPE};
use uuid::Uuid;

use crate::{property::EnumVariant, Entity};

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

pub fn add_entity<E: Entity>(value: Option<&E>) -> Markup {
    let form_id = &Uuid::new_v4().to_string();
    let ctx = FormRenderContext { form_id };
    html! {
        main {
            h1 {"Erstelle neues " (E::name().to_case(Case::Title))}
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
