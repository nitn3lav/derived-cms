use std::fmt::Debug;

pub use derived_cms_derive::Input;
use i18n_embed::fluent::FluentLanguageLoader;
use maud::Markup;

use crate::{context::ContextTrait, render::FormRenderContext};

/// A property of an entity or nested within another property that can be input in a HTML form
pub trait Input<S: ContextTrait>: Debug {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup;
}

/// object safe trait that is automatically implemented for [`Option<T>`] where `T` implements [`Input`]
pub trait DynInput<S: ContextTrait>: Debug {
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup;
}

impl<T: Input<S>, S: ContextTrait> DynInput<S> for Option<&T> {
    fn render_input(
        &self,
        name: &str,
        name_human: &str,
        required: bool,
        ctx: &FormRenderContext<'_, S>,
        i18n: &FluentLanguageLoader,
    ) -> Markup {
        Input::render_input(self.as_deref(), name, name_human, required, ctx, i18n)
    }
}

/// a dynamic reference to an [`Input`] and it's name
#[derive(Debug)]
pub struct InputInfo<'a, S: ContextTrait> {
    pub name: &'a str,
    pub name_human: &'a str,
    pub value: Box<dyn DynInput<S> + 'a>,
}
