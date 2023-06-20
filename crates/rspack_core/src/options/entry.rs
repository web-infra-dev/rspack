use indexmap::IndexMap;

use crate::{ChunkLoading, DependencyId, PublicPath};

pub type Entry = IndexMap<String, EntryData>;

pub type EntryItem = Vec<String>;

#[derive(Debug, Clone)]
pub struct EntryDescription {
  pub import: EntryItem,
  pub runtime: Option<String>,
  pub chunk_loading: Option<ChunkLoading>,
  pub public_path: Option<PublicPath>,
}

#[derive(Debug)]
pub struct EntryData {
  pub dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EntryOptions {
  pub runtime: Option<String>,
  pub chunk_loading: Option<ChunkLoading>,
  pub public_path: Option<PublicPath>,
}
