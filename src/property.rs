use std::fmt::Debug;

use chrono::{DateTime, TimeZone};
use derive_more::{Display, From, FromStr, Into};
use maud::{html, Markup, PreEscaped};
use serde::{Deserialize, Serialize};
use sqlx::Database;
use uuid::Uuid;

pub use derived_cms_derive::Property;

use crate::render::FormRenderContext;

/// A property of an entity or nested within another property that can be input in a HTML form
pub trait Property: Debug {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String>;
}

/// object safe trait that is automatically implemented for [`Option<T>`] where `T` implements [`Property`]
pub trait DynProperty: Debug {
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
#[derive(Debug)]
pub struct PropertyInfo<'a> {
    pub name: &'a str,
    pub value: Box<dyn DynProperty + 'a>,
}

#[derive(Debug)]
pub struct EnumVariant<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub content: Option<PropertyInfo<'a>>,
}

#[derive(
    Clone, Debug, Default, Display, From, FromStr, Into, PartialEq, Deserialize, Serialize,
)]
#[serde(transparent)]
pub struct Text(pub String);

impl<'r, DB: Database> sqlx::Decode<'r, DB> for Text
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(<String as sqlx::Decode<DB>>::decode(value)?))
    }
}

impl<DB: Database> sqlx::Type<DB> for Text
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}

impl<'r, DB: Database> sqlx::Encode<'r, DB> for Text
where
    String: sqlx::Encode<'r, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'r>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        sqlx::Encode::<'_, DB>::encode(&self.0, buf)
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

#[derive(
    Clone, Debug, Default, Display, From, FromStr, Into, PartialEq, Deserialize, Serialize,
)]
#[serde(transparent)]
pub struct Markdown(pub String);

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

impl<'r, DB: Database> sqlx::Decode<'r, DB> for Markdown
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(<String as sqlx::Decode<DB>>::decode(value)?))
    }
}

impl<DB: Database> sqlx::Type<DB> for Markdown
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}

impl<DB: Database> sqlx::Encode<'static, DB> for Markdown
where
    for<'a> String: sqlx::Encode<'a, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        sqlx::Encode::<'_, DB>::encode(&self.0, buf)
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
            input type="datetime-local" id=(input_id) class="cms-datetime-input" {}
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
        let list_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let name_regex = regex::escape(name);
        html! {
            div class="cms-list-input" id=(list_id) {
                @if let Some(v) = value {
                    @for (i, v) in v.iter().enumerate() {
                        fieldset class="cms-list-element" {
                            (Property::render_input(Some(v), &format!("{name}[{i}]"), name_human, ctx))
                        }
                    }
                }
                fieldset id=(template_id) class="cms-list-element" {
                    (Property::render_input(Option::<&T>::None, &format!("{name}[0]"), name_human, ctx))
                }
                button id=(btn_id) {"+"}
                script type="module" {(PreEscaped(format!(r#"
const btn = document.getElementById("{btn_id}");
const list = document.getElementById("{list_id}");
const template = document.getElementById("{template_id}");
template.remove();
btn.addEventListener("click", (e) => {{
    let el = template.cloneNode(true);
    el.removeAttribute("id");
    setIndex(el, list.childElementCount - 2)
    list.insertBefore(el, btn);
    e.preventDefault();
}});
function setIndex(el, i) {{
    for (const e of el.querySelectorAll("[name]")) {{
        e.name = e.name.replace(/^{name_regex}\[[0-9]*\]/, "{name}["+i+"]")
    }}
    for (const e of el.querySelectorAll("[id]")) {{
        e.id = e.id.replace(/^{name_regex}\[[0-9]*\]/, "{name}["+i+"]")
    }}
    for (const e of el.querySelectorAll("[for]")) {{
        e.attributes.for.value = e.attributes.for.value.replace(/^{name_regex}\[[0-9]*\]/, "{name}["+i+"]")
    }}
}}
                "#).trim()))}
            }
        }
    }
}

impl<T: Property> Property for ormlite::types::Json<T> {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> PreEscaped<String> {
        T::render_input(value.map(|v| &v.0), name, name_human, ctx)
    }
}
