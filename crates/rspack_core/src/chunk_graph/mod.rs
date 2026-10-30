use rspack_collections::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  AsyncDependenciesBlockIdentifierMap, ChunkGroupUkey, ChunkUkey, Compilation, DependenciesBlock,
  GroupOptions, ModuleIdentifier, RuntimeSpec,
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
  /// Current physical loading plans for dependency blocks in the given runtime.
  pub fn get_block_chunks(
    &self,
    compilation: &Compilation,
    runtime: &RuntimeSpec,
  ) -> std::collections::BTreeMap<String, Vec<chunk_graph_chunk::ChunkId>> {
    let graph = &compilation.build_chunk_graph_artifact;
    let mg = compilation.get_module_graph();
    let mut blocks = std::collections::BTreeMap::new();
    // Concatenated modules own their inner modules' blocks, but the blocks keep
    // their original parent identifiers. Use the module actually in the chunks.
    for &module in self.chunk_graph_module_by_module_identifier.keys() {
      let Some(parent) = mg.module_by_identifier(&module) else {
        continue;
      };
      if parent.get_blocks().is_empty()
        || !self
          .get_module_runtimes_iter(module, &graph.chunk_by_ukey)
          .any(|r| r.iter().any(|name| runtime.contains(name)))
      {
        continue;
      }
      let mut queue = parent.get_blocks().to_vec();
      while let Some(id) = queue.pop() {
        let block = mg.block_by_id_expect(&id);
        // Async entries start their own runtime instead of loading through `eg`.
        if matches!(block.get_group_options(), Some(GroupOptions::Entrypoint(_))) {
          continue;
        }
        queue.extend_from_slice(block.get_blocks());
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
        blocks.insert(block.runtime_id(compilation), chunks);
      }
    }
    blocks
  }

  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
