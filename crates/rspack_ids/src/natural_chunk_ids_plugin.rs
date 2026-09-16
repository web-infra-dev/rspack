use itertools::Itertools;
use rspack_core::{Chunk, CompilationChunkIds, Plugin, incremental::IncrementalPasses};
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{
  NaturalChunkCompareCache, assign_ascending_chunk_ids, compare_chunks_natural,
};

#[plugin]
#[derive(Debug, Default)]
pub struct NaturalChunkIdsPlugin;

#[plugin_hook(CompilationChunkIds for NaturalChunkIdsPlugin)]
async fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS | IncrementalPasses::MODULES_HASHES,
    "NaturalChunkIdsPlugin (optimization.chunkIds = \"natural\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) && let Some(diagnostic) = diagnostic
  {
    compilation.extend_diagnostics([diagnostic]);
  }

  let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_graph.chunks;
  let module_ids = &compilation.module_ids_artifact;
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let mut chunk_compare_cache = NaturalChunkCompareCache::default();

  let chunks = chunk_by_ukey
    .values()
    .map(|chunk| chunk as &Chunk)
    .sorted_unstable_by(|a, b| {
      compare_chunks_natural(
        chunk_graph,
        &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        module_ids,
        a,
        b,
        &mut chunk_compare_cache,
      )
    })
    .map(|chunk| chunk.ukey())
    .collect::<Vec<_>>();

  if !chunks.is_empty() {
    assign_ascending_chunk_ids(
      &chunks,
      &mut compilation.build_chunk_graph_artifact.chunk_graph.chunks,
    );
  }

  Ok(())
}

impl Plugin for NaturalChunkIdsPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> rspack_error::Result<()> {
    ctx.compilation_hooks.chunk_ids.tap(chunk_ids::new(self));
    Ok(())
  }
}
