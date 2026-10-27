//! Consumer check for candidates updated by committed placements.

use rspack_core::{ChunkGroupUkey, ChunkUkey, Compilation};
use rustc_hash::FxHashSet;

use crate::common::ModuleChunkMap;

/// Each replacement copy must retain the union of the consumers of its removed
/// copies. Also check existing target modules when extending a reused target.
pub(super) fn placement_preserves_copies(
  placed: &ModuleChunkMap,
  source_chunks: &FxHashSet<ChunkUkey>,
  target: ChunkUkey,
  compilation: &Compilation,
) -> bool {
  let artifact = &compilation.build_chunk_graph_artifact;
  let target_chunk = artifact.chunk_by_ukey.expect_get(&target);
  let consumers = |chunks: &FxHashSet<ChunkUkey>| -> FxHashSet<ChunkGroupUkey> {
    chunks
      .iter()
      .flat_map(|chunk| {
        artifact
          .chunk_by_ukey
          .expect_get(chunk)
          .groups()
          .iter()
          .copied()
      })
      .collect()
  };
  let mut after = consumers(source_chunks);
  after.extend(target_chunk.groups().iter().copied());
  let target_modules = artifact.chunk_graph.get_chunk_modules_identifier(&target);
  for module in target_modules {
    let mut before = placed.get(module).map(&consumers).unwrap_or_default();
    before.extend(target_chunk.groups().iter().copied());
    if before != after {
      return false;
    }
  }
  // Existing target modules have already been checked. New shared modules all
  // have the same source chunks, so check their consumers only once.
  match placed {
    ModuleChunkMap::Shared { modules, chunks } => {
      (!target_modules.is_empty() || !modules.is_empty())
        && (modules.is_subset(target_modules) || consumers(chunks) == after)
    }
    ModuleChunkMap::ByModule(modules) => {
      (!target_modules.is_empty() || !modules.is_empty())
        && modules
          .iter()
          .all(|(module, chunks)| target_modules.contains(module) || consumers(chunks) == after)
    }
  }
}
