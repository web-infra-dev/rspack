use itertools::Itertools;
use rspack_collections::DatabaseItem;
use rspack_core::{
  incremental::IncrementalPasses, ApplyContext, Chunk, CompilationChunkIds, CompilerOptions,
  Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{assign_ascending_chunk_ids, compare_chunks_natural};

#[plugin]
#[derive(Debug, Default)]
pub struct NaturalChunkIdsPlugin;

#[plugin_hook(CompilationChunkIds for NaturalChunkIdsPlugin)]
async fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::CHUNK_IDS,
    "NaturalChunkIdsPlugin (optimization.chunkIds = \"natural\")",
    "it requires calculating the id of all the chunks, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.chunk_ids_artifact.clear();
  }

  let module_ids = &compilation.module_ids_artifact;
  let chunk_graph = &compilation.chunk_graph;
  let module_graph = &compilation.get_module_graph();

  let chunks = compilation
    .chunk_by_ukey
    .values()
    .map(|chunk| chunk as &Chunk)
    .sorted_unstable_by(|a, b| {
      compare_chunks_natural(
        chunk_graph,
        module_graph,
        &compilation.chunk_group_by_ukey,
        module_ids,
        a,
        b,
      )
    })
    .map(|chunk| chunk.ukey())
    .collect::<Vec<_>>();

  if !chunks.is_empty() {
    assign_ascending_chunk_ids(&chunks, compilation);
  }

  Ok(())
}

impl Plugin for NaturalChunkIdsPlugin {
  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .chunk_ids
      .tap(chunk_ids::new(self));
    Ok(())
  }
}
