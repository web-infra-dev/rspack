use rspack_collections::IdentifierMap;
#[cfg(any(allocative, feature = "allocative"))]
use rspack_util::allocative;
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

#[cfg(any(allocative, feature = "allocative"))]
impl allocative::Allocative for ChunkGraph {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let module_count = self.chunk_graph_module_by_module_identifier.len();
    visitor.visit_field_with(
      allocative::Key::new("module_graph_entries"),
      std::mem::size_of_val(&self.chunk_graph_module_by_module_identifier)
        + self.chunk_graph_module_by_module_identifier.capacity()
          * std::mem::size_of::<ModuleIdentifier>()
        + self
          .chunk_graph_module_by_module_identifier
          .capacity()
          .saturating_sub(module_count)
          * std::mem::size_of::<ChunkGraphModule>(),
      |visitor| {
        for module in self.chunk_graph_module_by_module_identifier.values() {
          allocative::Allocative::visit(module, visitor);
        }
      },
    );
    let chunk_count = self.chunk_graph_chunk_by_chunk_ukey.len();
    visitor.visit_field_with(
      allocative::Key::new("chunk_graph_chunk_entries"),
      std::mem::size_of_val(&self.chunk_graph_chunk_by_chunk_ukey)
        + self.chunk_graph_chunk_by_chunk_ukey.capacity() * std::mem::size_of::<ChunkUkey>()
        + self
          .chunk_graph_chunk_by_chunk_ukey
          .capacity()
          .saturating_sub(chunk_count)
          * std::mem::size_of::<ChunkGraphChunk>(),
      |visitor| {
        for chunk in self.chunk_graph_chunk_by_chunk_ukey.values() {
          allocative::Allocative::visit(chunk, visitor);
        }
      },
    );
    visitor.visit_field_with(
      allocative::Key::new("block_chunk_group_index"),
      std::mem::size_of_val(&self.block_to_chunk_group_ukey)
        + self.block_to_chunk_group_ukey.capacity()
          * std::mem::size_of::<(crate::AsyncDependenciesBlockIdentifier, ChunkGroupUkey)>(),
      |visitor| {
        visitor.visit_simple(
          allocative::Key::new("entries"),
          self.block_to_chunk_group_ukey.len() * std::mem::size_of::<ChunkGroupUkey>(),
        );
      },
    );
    // Runtime names are generated labels. Do not include their string contents in the snapshot.
    visitor.visit_simple(
      allocative::Key::new("runtime_id_index"),
      self.runtime_ids.capacity() * std::mem::size_of::<(String, Option<String>)>()
        + self
          .runtime_ids
          .iter()
          .map(|(key, value)| key.capacity() + value.as_ref().map_or(0, String::capacity))
          .sum::<usize>(),
    );
    visitor.exit();
  }
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
