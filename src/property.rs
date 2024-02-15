use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

use chrono::{DateTime, TimeZone};
use derive_more::{Deref, DerefMut, Display, From, FromStr, Into};
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use maud::{html, Markup, PreEscaped};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    self as derived_cms, context::ContextTrait, input::InputInfo, render::FormRenderContext,
    Column, Input, DB,
};

#[derive(Debug)]
pub struct EnumVariant<'a, S: ContextTrait> {
    pub name: &'a str,
    pub value: &'a str,
    pub content: Option<InputInfo<'a, S>>,
}

/********
 * Text *
 ********/

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Display,
    From,
    FromStr,
    Into,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    Column,
)]
#[serde(transparent)]
pub struct Text(pub String);

impl TS for Text {
    fn name() -> String {
        "string".to_string()
    }

    fn dependencies() -> Vec<ts_rs::Dependency>
    where
        Self: 'static,
    {
        Vec::new()
    }

    fn transparent() -> bool {
        true
    }
}

impl<'r> sqlx::Decode<'r, DB> for Text
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(<String as sqlx::Decode<DB>>::decode(value)?))
    }
}

impl sqlx::Type<DB> for Text
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}

impl<'r> sqlx::Encode<'r, DB> for Text
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

impl<S: ContextTrait> Input<S> for Text {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        _ctx: &FormRenderContext<'_, S>,
        _i18n: &FluentLanguageLoader,
    ) -> Markup {
        html! {
            input type="text" name=(name) placeholder=(name_human) class="cms-text-input" value=[value] required[required] {}
        }
    }
}

/************
 * Markdown *
 ************/

#[derive(
    Clone,
    Debug,
    Default,
    Deref,
    DerefMut,
    Display,
    From,
    FromStr,
    Into,
    PartialEq,
    Eq,
    Hash,
    Deserialize,
    Serialize,
    Column,
)]
#[serde(transparent)]
pub struct Markdown(pub String);

impl TS for Markdown {
    fn name() -> String {
        "string".to_string()
    }

    fn dependencies() -> Vec<ts_rs::Dependency>
    where
        Self: 'static,
    {
        Vec::new()
    }

    fn transparent() -> bool {
        true
    }
}

impl<S: ContextTrait> Input<S> for Markdown {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        _ctx: &FormRenderContext<'_, S>,
        _i18n: &FluentLanguageLoader,
    ) -> Markup {
        let id = Uuid::new_v4();
        html! {
            div class="cms-markdown-editor" {
                textarea
                    name=(name)
                    placeholder=(name_human)
                    required[required]
                    id=(id)
                    onmount="new EasyMDE({ element: this })" {
                    (value.map(|v| v.0.as_ref()).unwrap_or(""))
                }
                script src="https://cdn.jsdelivr.net/npm/easymde/dist/easymde.min.js" {}
            }
        }
    }
}
impl<'r> sqlx::Decode<'r, DB> for Markdown
where
    String: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(<String as sqlx::Decode<DB>>::decode(value)?))
    }
}
impl sqlx::Type<DB> for Markdown
where
    String: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }
}
impl<'r> sqlx::Encode<'r, DB> for Markdown
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

/************
 * DateTime *
 ************/

impl<Tz: TimeZone, S: ContextTrait> Input<S> for DateTime<Tz>
where
    for<'de> DateTime<Tz>: Deserialize<'de>,
{
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        _i18n: &FluentLanguageLoader,
    ) -> Markup {
        let input_id = Uuid::new_v4();
        let hidden_id = Uuid::new_v4();
        html! {
            input type="datetime-local" id=(input_id) class="cms-datetime-input" {}
            input type="hidden" name=(name) id=(hidden_id) value=[value.map(|v|v.to_rfc3339())] required[required] {}
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
impl<Tz: TimeZone> Column for DateTime<Tz>
where
    Tz::Offset: std::fmt::Display,
{
    fn render(&self, _i18n: &FluentLanguageLoader) -> Markup {
        html! {
            time datetime=(self.to_rfc3339()) {
                (self.to_string())
            }
        }
    }
}

/********
 * bool *
 ********/

impl<S: ContextTrait> Input<S> for bool {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        _required: bool,
        _ctx: &FormRenderContext<'_, S>,
        _i18n: &FluentLanguageLoader,
    ) -> Markup {
        html! {
            input type="checkbox" name=(name) value="true" checked[*value.unwrap_or(&false)] {}
        }
    }
}
impl Column for bool {
    fn render(&self, _i18n: &FluentLanguageLoader) -> Markup {
        html! {
            input type="checkbox" disabled checked[*self] {}
        }
    }
}

/**********
 * Vec<T> *
 **********/

impl<T: Input<S>, S: ContextTrait> Input<S> for Vec<T> {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        let btn_id = Uuid::new_v4();
        let list_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let name_regex = regex::escape(name);
        html! {
            div class="cms-list-input" id=(list_id) {
                @if let Some(v) = value {
                    @for (i, v) in v.iter().enumerate() {
                        fieldset class="cms-list-element" {
                            (Input::render_input(Some(v), &format!("{name}[{i}]"), name_human, required, ctx, i18n))
                        }
                    }
                }
                fieldset id=(template_id) class="cms-list-element" style="display: none" onmount="return true" {
                    (Input::render_input(Option::<&T>::None, &format!("{name}[]"), name_human, required, ctx, i18n))
                }
                button id=(btn_id) {"+"}
                script type="module" {(PreEscaped(format!(r#"
const btn = document.getElementById("{btn_id}");
const list = document.getElementById("{list_id}");
const template = document.getElementById("{template_id}");
template.remove();
template.removeAttribute("style");
btn.addEventListener("click", (e) => {{
    e.preventDefault();
    let el = template.cloneNode(true);
    el.removeAttribute("id");
    setIndex(el, list.childElementCount - 2)
    list.insertBefore(el, btn);
    callOnMountRecursive(el);
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

/**********
 * Option *
 **********/

impl<T: Input<S>, S: ContextTrait> Input<S> for Option<T> {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        _required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        let value = match value {
            Some(v) => v.as_ref(),
            None => None,
        };
        T::render_input(value, name, name_human, false, ctx, i18n)
    }
}

impl<T: Column> Column for Option<T> {
    fn render(&self, i18n: &FluentLanguageLoader) -> Markup {
        match self {
            Some(v) => v.render(i18n),
            None => html!(),
        }
    }
}

/********
 * Json *
 ********/

#[cfg(feature = "json")]
#[derive(
    Copy,
    Clone,
    Debug,
    Deref,
    DerefMut,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct Json<T: ?Sized>(pub T);

#[cfg(feature = "json")]
impl<T: TS> TS for Json<T> {
    fn name() -> String {
        "Json".to_string()
    }

    fn dependencies() -> Vec<ts_rs::Dependency>
    where
        Self: 'static,
    {
        Vec::new()
    }

    fn transparent() -> bool {
        true
    }

    fn name_with_type_args(args: Vec<String>) -> String {
        args[0].clone()
    }
}

#[cfg(feature = "json")]
impl<'r, T> sqlx::Decode<'r, DB> for Json<T>
where
    sqlx::types::Json<T>: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(
            <sqlx::types::Json<T> as sqlx::Decode<DB>>::decode(value)?.0,
        ))
    }
}
#[cfg(feature = "json")]
impl<T> sqlx::Type<DB> for Json<T>
where
    sqlx::types::Json<T>: sqlx::Type<DB>,
{
    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <sqlx::types::Json<T> as sqlx::Type<DB>>::type_info()
    }
}
#[cfg(feature = "json")]
impl<'q, T> sqlx::Encode<'q, DB> for Json<T>
where
    for<'a> sqlx::types::Json<&'a T>: sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx_core::encode::IsNull {
        <sqlx::types::Json<&T> as sqlx::Encode<'q, DB>>::encode(sqlx::types::Json(&self.0), buf)
    }
}

#[cfg(feature = "json")]
impl<T: Input<S>, S: ContextTrait> Input<S> for Json<T> {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        T::render_input(value.map(|v| &v.0), name, name_human, required, ctx, i18n)
    }
}
#[cfg(feature = "json")]
impl<T: Column> Column for Json<T> {
    fn render(&self, i18n: &FluentLanguageLoader) -> Markup {
        self.0.render(i18n)
    }
}

/********
 * Uuid *
 ********/

impl Column for Uuid {
    fn render(&self, _i18n: &FluentLanguageLoader) -> Markup {
        html!((self))
    }
}

/********
 * File *
 ********/

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
pub struct File {
    /// name of the file created in `files_dir`
    id: Uuid,
    /// original filename
    name: String,
}

impl File {
    pub fn url(&self) -> String {
        format!("/uploads/{}/{}", self.id, self.name)
    }

    pub fn path(&self, uploads_dir: &Path) -> PathBuf {
        uploads_dir.join(self.id.to_string()).join(&self.name)
    }
}

impl<'de> Deserialize<'de> for File {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct File {
            id: Option<Uuid>,
            id_old: Option<Uuid>,
            name: Option<String>,
            name_old: Option<String>,
        }
        let f = File::deserialize(deserializer)?;
        let id =
            f.id.or(f.id_old)
                .map(Into::into)
                .ok_or(serde::de::Error::missing_field("id"))?;
        let name = f
            .name
            .or(f.name_old)
            .map(Into::into)
            .ok_or(serde::de::Error::missing_field("name"))?;
        // TODO: check if file exists
        Ok(Self { id, name })
    }
}

impl<S: ContextTrait> Input<S> for File {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        required: bool,
        _ctx: &FormRenderContext<'_, S>,
        _i18n: &FluentLanguageLoader,
    ) -> Markup {
        html! {
            @if let Some(v) = value {
                input type="hidden" name=(format!("{name}[id_old]")) value=(v.id) {}
                input type="hidden" name=(format!("{name}[name_old]")) value=(v.name) {}
            }
            input type="file" name=(name) required[required && value.is_none()] {}
        }
    }
}

impl Column for File {
    fn render(&self, _i18n: &FluentLanguageLoader) -> Markup {
        html! {
            a href=(self.url()) {
                (self.name)
            }
        }
    }
}

/*********
 * Image *
 *********/

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, TS)]
pub struct Image {
    #[serde(flatten)]
    pub file: File,
    pub alt_text: Option<String>,
}

impl<S: ContextTrait> Input<S> for Image {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        _name_human: &str,
        required: bool,
        _ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        html! {
            fieldset class="cms-image cms-prop-group" {
                @if let Some(v) = value {
                    input type="hidden" name=(format!("{name}[id_old]")) value=(v.file.id) {}
                    input type="hidden" name=(format!("{name}[name_old]")) value=(v.file.name) {}
                }
                input type="file" accept="image/*" name=(name) required[required && value.is_none()] {}
                input
                    type="text"
                    name=(format!("{name}[alt_text]"))
                    placeholder=(fl!(i18n, "image-alt-text"))
                    class="cms-text-input cms-prop-container"
                    value=[value.map(|v| v.alt_text.as_deref().unwrap_or_default())] {}
            }
        }
    }
}

impl Column for Image {
    fn render(&self, _i18n: &FluentLanguageLoader) -> Markup {
        html! {
            a href=(self.file.url()) {
                (self.file.name)
            }
            @if let Some(alt_text) = &self.alt_text {
                " (" (alt_text) ")"
            }
        }
    }
}
