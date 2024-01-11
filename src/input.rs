use std::fmt::Debug;

use maud::Markup;

use crate::render::FormRenderContext;

pub use derived_cms_derive::Input;

/// A property of an entity or nested within another property that can be input in a HTML form
pub trait Input: Debug {
    fn render_input(
        value: Option<&Self>,
        name: &str,
        name_human: &str,
        ctx: &FormRenderContext,
    ) -> Markup;
}

/// object safe trait that is automatically implemented for [`Option<T>`] where `T` implements [`Input`]
pub trait DynInput: Debug {
    fn render_input(&self, name: &str, name_human: &str, ctx: &FormRenderContext) -> Markup;
}

impl<T: Input> DynInput for Option<&T> {
    fn render_input(&self, name: &str, name_human: &str, ctx: &FormRenderContext) -> Markup {
        Input::render_input(self.as_deref(), name, name_human, ctx)
    }
}

/// a dynamic reference to an [`Input`] and it's name
#[derive(Debug)]
pub struct InputInfo<'a> {
    pub name: &'a str,
    pub value: Box<dyn DynInput + 'a>,
}
