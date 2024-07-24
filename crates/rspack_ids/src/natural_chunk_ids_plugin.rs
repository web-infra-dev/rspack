use itertools::Itertools;
use rspack_core::{
  ApplyContext, Chunk, CompilationChunkIds, CompilerOptions, Plugin, PluginContext,
};
use rspack_hook::{plugin, plugin_hook};

use crate::id_helpers::{assign_ascending_chunk_ids, compare_chunks_natural};

#[plugin]
#[derive(Debug, Default)]
pub struct NaturalChunkIdsPlugin;

#[plugin_hook(CompilationChunkIds for NaturalChunkIdsPlugin)]
fn chunk_ids(&self, compilation: &mut rspack_core::Compilation) -> rspack_error::Result<()> {
  let chunk_graph = &compilation.chunk_graph;
  let module_graph = &compilation.get_module_graph();

  let chunks = compilation
    .chunk_by_ukey
    .values()
    .map(|chunk| chunk as &Chunk)
    .sorted_unstable_by(|a, b| compare_chunks_natural(chunk_graph, module_graph, a, b))
    .map(|chunk| chunk.ukey)
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
    _options: &mut CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .chunk_ids
      .tap(chunk_ids::new(self));
    Ok(())
  }
}
