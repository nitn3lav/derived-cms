use std::fmt::Display;

use chrono::{DateTime, TimeZone};
use maud::{html, Markup, PreEscaped};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A property of an entity or nested within another property that can be input in a HTML form
pub trait Property {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String>;
}

/// object safe trait that is automatically implemented for [`Option<T>`] where `T` implements [`Property`]
pub trait DynProperty {
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String>;
}

impl<T> DynProperty for Option<&T>
where
    T: Property,
{
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        Property::render_input(self.as_deref(), name, name_human, ctx)
    }
}

/// a dynamic reference to a property and it's name
pub struct PropertyInfo<'a> {
    pub name: &'a str,
    pub value: Box<dyn DynProperty + 'a>,
}

#[non_exhaustive]
pub struct FormRenderContext<'a> {
    /// unique id of the HTML form element
    pub form_id: &'a str,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Text(pub String);

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Property for Text {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        _ctx: &FormRenderContext,
    ) -> Markup {
        html! {
            input type="text" name=(name) placeholder=(name_human) class="cms-text-input" value=[value] {}
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Markdown(pub String);

impl Display for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Property for Markdown {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        _ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        html! {
            div class="markdown-buttons" {

            }
            textarea name=(name) placeholder=(name_human) value=[value] {}
        }
    }
}

impl<Tz: TimeZone> Property for DateTime<Tz>
where
    for<'de> DateTime<Tz>: Deserialize<'de>,
    Tz::Offset: std::fmt::Display,
{
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        let input_id = Uuid::new_v4();
        let hidden_id = Uuid::new_v4();
        html! {
            input type="datetime-local" id=(input_id) {}
            input type="hidden" name=(name) id=(hidden_id) value=[value] {}
            script type="module" {(PreEscaped(format!(r#"
const input = document.getElementById("{input_id}");
const hidden = document.getElementById("{hidden_id}");
const d = new Date(hidden.value);
input.value = `${{d.getFullYear()}}-${{(d.getMonth()+1).toString().padStart(2, '0')}}-${{d.getDate().toString().padStart(2, '0')}}T${{d.getHours().toString().padStart(2, '0')}}:${{d.getMinutes().toString().padStart(2, '0')}}`;
document.getElementById("{}").addEventListener("submit", () => {{
    hidden.value = new Date(input.value).toISOString();
}});
            "#, ctx.form_id).trim()))}
            noscript {
                "It appears that JavaScript is disabled. JavaScript is required to set dates in your current timezone. Please enter dates in UTC (Coordinated universal time) instead."
            }
        }
    }
}

impl Property for bool {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        _ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        html! {
            input type="checkbox" name=(name) checked[*value.unwrap_or(&false)] {}
        }
    }
}

impl<T> Property for Vec<T>
where
    T: Property,
{
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        let btn_id = Uuid::new_v4();
        let table_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let name_regex = regex::escape(name);
        html! {
            div class="cms-list-input" {
                table id=(table_id) class="cms-table" {
                    @if let Some(v) = value {
                        @for (i, v) in v.iter().enumerate() {
                            (Property::render_input(Some(v), &format!("{name}[{i}]"), name_human, ctx))
                        }
                    }
                }
                div id=(template_id) {
                    (Property::render_input(Option::<&T>::None, &format!("{name}[0]"), name_human, ctx))
                }
                button id=(btn_id) {"+"}
                script type="module" {(PreEscaped(format!(r#"
const btn = document.getElementById("{btn_id}");
const table = document.getElementById("{table_id}");
const template = document.getElementById("{template_id}");
template.remove();
btn.addEventListener("click", (e) => {{
    let el = template.cloneNode(true);
    setIndex(el, table.childElementCount)
    table.appendChild(el);
    e.preventDefault();
}});
function setIndex(el, i) {{
    for (const e of el.querySelectorAll("[name]")) {{
        e.name = e.name.replace(/^{name_regex}\[[0-9]*\]/, "{name}["+i+"]")
    }}
}}
                "#).trim()))}
            }
        }
    }
}
