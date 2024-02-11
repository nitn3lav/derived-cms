use std::fmt::Debug;

use i18n_embed::fluent::FluentLanguageLoader;
use maud::Markup;

use crate::render::FormRenderContext;

pub use derived_cms_derive::Input;

/// A property of an entity or nested within another property that can be input in a HTML form
pub trait Input: Debug {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext,
        i18n: &FluentLanguageLoader,
    ) -> Markup;
}

/// object safe trait that is automatically implemented for [`Option<T>`] where `T` implements [`Input`]
pub trait DynInput: Debug {
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext,
        i18n: &FluentLanguageLoader,
    ) -> Markup;
}

impl<T: Input> DynInput for Option<&T> {
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        Input::render_input(self.as_deref(), name, name_human, required, ctx, i18n)
    }
}

/// a dynamic reference to an [`Input`] and it's name
#[derive(Debug)]
pub struct InputInfo<'a> {
    pub name: &'a str,
    pub name_human: &'a str,
    pub value: Box<dyn DynInput + 'a>,
}
