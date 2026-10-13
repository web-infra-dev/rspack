use rspack_collections::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  AsyncDependenciesBlockIdentifierMap, ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier,
  RuntimeSpec,
};

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
  /// Current physical loading plans, optionally restricted to one runtime.
  pub fn get_async_chunk_groups(
    &self,
    compilation: &Compilation,
    runtime: Option<&RuntimeSpec>,
  ) -> std::collections::BTreeMap<String, Vec<chunk_graph_chunk::ChunkId>> {
    let graph = &compilation.build_chunk_graph_artifact;
    let mg = compilation.get_module_graph();
    mg.blocks()
      .values()
      .filter_map(|block| {
        if !self
          .chunk_graph_module_by_module_identifier
          .contains_key(block.parent())
        {
          return None;
        }
        if let Some(runtime) = runtime
          && !self
            .get_module_runtimes_iter(*block.parent(), &graph.chunk_by_ukey)
            .any(|r| r.iter().any(|name| runtime.contains(name)))
        {
          return None;
        }
        let chunks = self
          .get_block_chunk_group(&block.identifier(), &graph.chunk_group_by_ukey)
          .map(|group| {
            group
              .chunks
              .iter()
              .filter_map(|ukey| {
                let chunk = graph.chunk_by_ukey.expect_get(ukey);
                (!chunk.has_runtime(&graph.chunk_group_by_ukey))
                  .then(|| chunk.id().cloned())
                  .flatten()
              })
              .collect()
          })
          .unwrap_or_default();
        Some((block.runtime_id(compilation), chunks))
      })
      .collect()
  }

  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
