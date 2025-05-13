use indexmap::IndexSet;
use rspack_collections::{IdentifierMap, UkeyIndexMap, UkeyMap};
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{AsyncDependenciesBlockIdentifier, ChunkGroupUkey, ChunkUkey, ModuleIdentifier};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::{ChunkGraphChunk, ChunkSizeOptions};
pub use chunk_graph_module::{ChunkGraphModule, ModuleId};

#[derive(Debug, Clone, Default)]
pub struct ChunkLink {
  pub imports: UkeyIndexMap<ChunkUkey, IdentifierMap<IndexSet<Atom>>>,
  pub exports: IdentifierMap<HashSet<Atom>>,
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: HashMap<AsyncDependenciesBlockIdentifier, ChunkGroupUkey>,

  pub(crate) chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: UkeyMap<ChunkUkey, ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,

  // only used for esm output
  pub link: Option<UkeyMap<ChunkUkey, ChunkLink>>,
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
