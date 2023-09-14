// Port of https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/optimize/RemoveEmptyChunksPlugin.js

use rspack_core::{Compilation, Logger, Plugin};

#[derive(Debug)]
pub struct RemoveEmptyChunksPlugin;

impl RemoveEmptyChunksPlugin {
  fn remove_empty_chunks(&self, compilation: &mut Compilation) {
    let logger = compilation.get_logger(self.name());
    let start = logger.time("remove empty chunks");

    let chunk_graph = &mut compilation.chunk_graph;
    let mut empty_chunks = compilation
      .chunk_by_ukey
      .values_mut()
      .filter(|chunk| {
        chunk_graph.get_number_of_chunk_modules(&chunk.ukey) == 0
          && !chunk.has_runtime(&compilation.chunk_group_by_ukey)
          && chunk_graph.get_number_of_entry_modules(&chunk.ukey) == 0
      })
      .collect::<Vec<_>>();

    empty_chunks.iter_mut().for_each(|chunk| {
      chunk_graph.disconnect_chunk(chunk, &mut compilation.chunk_group_by_ukey);
    });
    let to_be_removed = empty_chunks
      .iter()
      .map(|chunk| chunk.ukey)
      .collect::<Vec<_>>();

    to_be_removed.iter().for_each(|ukey| {
      compilation.chunk_by_ukey.remove(ukey);
    });

    logger.time_end(start);
  }
}

#[async_trait::async_trait]
impl Plugin for RemoveEmptyChunksPlugin {
  fn name(&self) -> &'static str {
    "rspack.RemoveEmptyChunksPlugin"
  }

  async fn optimize_chunks(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    let compilation = args.compilation;
    self.remove_empty_chunks(compilation);
    Ok(())
  }
}
