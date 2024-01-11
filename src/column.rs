use std::fmt::Debug;

use maud::Markup;

pub use derived_cms_derive::Column;

/// A property of an entity that can be rendered as a column on the list page
pub trait Column: Debug {
    fn render(&self) -> Markup;
}
