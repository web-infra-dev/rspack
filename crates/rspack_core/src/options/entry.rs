use indexmap::IndexMap;

use crate::{ChunkLoading, DependencyId, EntryOptions, Filename, LibraryOptions, PublicPath};

pub type Entry = IndexMap<String, EntryData>;

pub type EntryItem = Vec<String>;

#[derive(Debug, Clone, Default)]
pub struct EntryDescription {
  pub import: Option<EntryItem>,
  pub runtime: Option<String>,
  pub chunk_loading: Option<ChunkLoading>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub filename: Option<Filename>,
  pub depend_on: Option<Vec<String>>,
  pub library: Option<LibraryOptions>,
}

impl<V> From<V> for EntryDescription
where
  V: Into<String>,
{
  fn from(value: V) -> Self {
    Self {
      import: Some(vec![value.into()]),
      ..Default::default()
    }
  }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EntryData {
  pub dependencies: Vec<DependencyId>,
  pub include_dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}

impl EntryData {
  pub fn all_dependencies(&self) -> impl Iterator<Item = &DependencyId> {
    self
      .dependencies
      .iter()
      .chain(self.include_dependencies.iter())
  }
}
