use indexmap::IndexMap;

use crate::{ChunkLoading, DependencyId, EntryOptions, Filename, PublicPath};

pub type Entry = IndexMap<String, EntryData>;

pub type EntryItem = Vec<String>;

#[derive(Debug, Clone)]
pub struct EntryDescription {
  pub import: EntryItem,
  pub runtime: Option<String>,
  pub chunk_loading: Option<ChunkLoading>,
  pub async_chunks: Option<bool>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub filename: Option<Filename>,
}

#[derive(Debug, Default, Clone)]
pub struct EntryData {
  pub dependencies: Vec<DependencyId>,
  pub include_dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}
