// Port of https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/optimize/RemoveEmptyChunksPlugin.js

use rspack_collections::DatabaseItem;
use rspack_core::{incremental::Mutation, Compilation, CompilationOptimizeChunks, Logger, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

#[plugin]
#[derive(Debug, Default)]
pub struct RemoveEmptyChunksPlugin;

impl RemoveEmptyChunksPlugin {
  fn remove_empty_chunks(&self, compilation: &mut Compilation) {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("remove empty chunks");

    let chunk_graph = &mut compilation.chunk_graph;
    let empty_chunks = compilation
      .chunk_by_ukey
      .values()
      .filter(|chunk| {
        chunk_graph.get_number_of_chunk_modules(&chunk.ukey()) == 0
          && !chunk.has_runtime(&compilation.chunk_group_by_ukey)
          && chunk_graph.get_number_of_entry_modules(&chunk.ukey()) == 0
      })
      .map(|chunk| chunk.ukey())
      .collect::<Vec<_>>();

    empty_chunks.iter().for_each(|chunk_ukey| {
      if let Some(mut chunk) = compilation.chunk_by_ukey.remove(chunk_ukey) {
        chunk_graph.disconnect_chunk(&mut chunk, &mut compilation.chunk_group_by_ukey);
        if let Some(mutations) = compilation.incremental.mutations_write() {
          mutations.add(Mutation::ChunkRemove { chunk: *chunk_ukey });
        }
      }
    });

    logger.time_end(start);
  }
}

#[plugin_hook(CompilationOptimizeChunks for RemoveEmptyChunksPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  self.remove_empty_chunks(compilation);
  Ok(None)
}

impl Plugin for RemoveEmptyChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.RemoveEmptyChunksPlugin"
  }

  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
