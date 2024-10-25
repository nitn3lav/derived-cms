use std::fmt::Debug;

pub use derived_cms_derive::Column;
use i18n_embed::fluent::FluentLanguageLoader;
use maud::Markup;

/// A property of an entity that can be rendered as a column on the list page
pub trait Column: Debug {
    fn render(&self, i18n: &FluentLanguageLoader) -> Markup;
}

#[derive(Clone, Debug)]
pub struct ColumnInfo {
    pub name: &'static str,
    /// whether the column is hidden by default
    pub hidden: bool,
}
