use rspack_collections::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{AsyncDependenciesBlockIdentifierMap, ChunkGroupUkey, ChunkUkey, ModuleIdentifier};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::{ChunkGraphChunk, ChunkIdMap, ChunkSizeOptions, IndexChunkIdMap};
pub use chunk_graph_module::{ChunkGraphModule, ModuleId, ModuleIdMap};

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: AsyncDependenciesBlockIdentifierMap<ChunkGroupUkey>,

  pub(crate) chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: HashMap<ChunkUkey, ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}

#[cfg(allocative)]
use rspack_util::allocative;

#[cfg(allocative)]
impl allocative::Allocative for ChunkGraph {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(
      allocative::Key::new("block_to_chunk_group_ukey"),
      &self.block_to_chunk_group_ukey,
    );
    visitor.visit_field(
      allocative::Key::new("chunk_graph_module_by_module_identifier"),
      &self.chunk_graph_module_by_module_identifier,
    );
    visitor.visit_field(allocative::Key::new("runtime_ids"), &self.runtime_ids);

    visitor.exit();
  }
}
