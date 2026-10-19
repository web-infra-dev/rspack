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
  let mut modules = artifact
    .chunk_graph
    .get_chunk_modules_identifier(&target)
    .clone();
  match placed {
    ModuleChunkMap::Shared {
      modules: moving, ..
    } => modules.extend(moving.iter().copied()),
    ModuleChunkMap::ByModule(moving) => modules.extend(moving.keys().copied()),
  }
  let mut checked_shared = false;
  for module in &modules {
    let in_target = artifact.chunk_graph.is_module_in_chunk(module, target);
    if let Some(sources) = placed.get(module) {
      if matches!(placed, ModuleChunkMap::Shared { .. }) && !in_target {
        if checked_shared {
          continue;
        }
        checked_shared = true;
      }
      let mut before = consumers(sources);
      if in_target {
        before.extend(target_chunk.groups().iter().copied());
      }
      if before != after {
        return false;
      }
    } else {
      let before: FxHashSet<_> = target_chunk.groups().iter().copied().collect();
      if before != after {
        return false;
      }
    }
  }
  !modules.is_empty()
}
