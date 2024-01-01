use std::fmt::Display;

use generic_array::{ArrayLength, GenericArray};
use maud::Markup;
use serde::{Deserialize, Serialize};

use crate::property::PropertyInfo;

pub use derived_cms_derive::Entity;

pub trait Entity: for<'de> Deserialize<'de> + Serialize {
    type NumberOfColumns: ArrayLength;

    fn name() -> impl Display + AsRef<str>;
    fn name_plural() -> impl Display + AsRef<str>;

    fn render_column_values(&self) -> GenericArray<Markup, Self::NumberOfColumns>;
    fn properties(value: Option<&Self>) -> impl IntoIterator<Item = PropertyInfo<'_>>;
}
