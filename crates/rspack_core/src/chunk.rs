use hashbrown::HashSet;
use tracing::instrument;

use crate::{ModuleGraph, ModuleGraphModule};

#[derive(Debug)]
pub struct Chunk {
  pub id: String,
  pub(crate) module_uris: HashSet<String>,
  pub(crate) entry_uri: String,
  pub kind: ChunkKind,
}

impl Chunk {
  pub fn new(id: String, entry_uri: String, kind: ChunkKind) -> Self {
    Self {
      id,
      module_uris: Default::default(),
      entry_uri,
      kind,
    }
  }

  #[instrument]
  pub fn ordered_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
    let mut ordered = self
      .module_uris
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect::<Vec<_>>();
    ordered.sort_by_key(|m| m.exec_order);
    ordered
  }
}

#[derive(Debug)]
pub enum ChunkKind {
  Entry { name: String },
  Normal,
  // TODO: support it.
  // Initial,
}

impl ChunkKind {
  pub fn is_entry(&self) -> bool {
    matches!(self, ChunkKind::Entry { .. })
  }
  pub fn is_normal(&self) -> bool {
    matches!(self, ChunkKind::Normal)
  }
}
