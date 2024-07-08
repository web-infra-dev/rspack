use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{AsyncDependenciesBlockIdentifier, ModuleIdentifier};
use crate::{ChunkGroupUkey, ChunkUkey};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::ChunkGraphChunk;
pub use chunk_graph_chunk::ChunkSizeOptions;
pub use chunk_graph_module::ChunkGraphModule;

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  pub split_point_module_identifier_to_chunk_ukey: IdentifierMap<ChunkUkey>,

  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: HashMap<AsyncDependenciesBlockIdentifier, ChunkGroupUkey>,

  pub chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.get_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
