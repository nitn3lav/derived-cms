use std::path::{Path, PathBuf};

use axum::extract::FromRef;

/// Trait implemented by the context available in all endpoints using [`axum::extract::State`].
pub trait ContextTrait: Clone + Send + Sync + 'static {
    type Ext: ContextExt<Self>;

    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>>;
    fn uploads_dir(&self) -> &Path;
    fn ext(&self) -> &Self::Ext;
}

#[derive(Debug)]
pub struct Context<T: ContextExt<Self>> {
    pub(crate) names_plural: Vec<&'static str>,
    pub(crate) uploads_dir: PathBuf,
    pub(crate) ext: T,
}
impl<E: ContextExt<Self>> Clone for Context<E> {
    fn clone(&self) -> Self {
        Self {
            names_plural: self.names_plural.clone(),
            uploads_dir: self.uploads_dir.clone(),
            ext: self.ext.clone(),
        }
    }
}
impl<E: ContextExt<Self> + 'static> ContextTrait for Context<E> {
    type Ext = E;

    fn names_plural(&self) -> impl Iterator<Item = impl AsRef<str>> {
        self.names_plural.iter()
    }
    fn uploads_dir(&self) -> &Path {
        &self.uploads_dir
    }
    fn ext(&self) -> &E {
        &self.ext
    }
}

impl FromRef<Context<()>> for () {
    fn from_ref(_input: &Context<()>) -> Self {}
}

pub trait ContextExt<Ctx>: Clone + Send + Sync {}

impl<Ctx, T: Send + Sync + 'static> ContextExt<Ctx> for T where T: Clone {}
