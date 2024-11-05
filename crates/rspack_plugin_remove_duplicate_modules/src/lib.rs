use std::sync::Arc;

use rspack_core::{
  Chunk, ChunkUkey, Compilation, CompilationOptimizeChunks, ModuleIdentifier, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;

#[derive(Debug)]
#[plugin]
pub struct RemoveDuplicateModulesPlugin {}

impl std::default::Default for RemoveDuplicateModulesPlugin {
  fn default() -> Self {
    Self {
      inner: Arc::new(RemoveDuplicateModulesPluginInner {}),
    }
  }
}

#[plugin_hook(CompilationOptimizeChunks for RemoveDuplicateModulesPlugin)]
fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.chunk_graph;

  let mut chunk_map: FxHashMap<Vec<ChunkUkey>, Vec<ModuleIdentifier>> = FxHashMap::default();

  for identifier in module_graph.modules().keys() {
    let chunks = chunk_graph.get_module_chunks(*identifier);
    let mut sorted_chunks = chunks.iter().copied().collect::<Vec<_>>();
    sorted_chunks.sort();
    chunk_map
      .entry(sorted_chunks)
      .or_default()
      .push(*identifier);
  }

  for (chunks, modules) in chunk_map {
    if chunks.len() <= 1 {
      continue;
    }

    // split chunks from original chunks and create new chunk
    let mut new_chunk = Chunk::new(None, rspack_core::ChunkKind::Normal);
    new_chunk.chunk_reason = Some("modules are shared across multiple chunks".into());
    let new_chunk_ukey = new_chunk.ukey;
    compilation.chunk_graph.add_chunk(new_chunk_ukey);

    for chunk_ukey in &chunks {
      let origin = compilation.chunk_by_ukey.expect_get_mut(chunk_ukey);
      origin.split(&mut new_chunk, &mut compilation.chunk_group_by_ukey);
    }

    compilation.chunk_by_ukey.add(new_chunk);

    for m in modules {
      for chunk_ukey in &chunks {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(chunk_ukey, m);
      }

      compilation
        .chunk_graph
        .connect_chunk_and_module(new_chunk_ukey, m);
    }
  }

  Ok(None)
}

impl Plugin for RemoveDuplicateModulesPlugin {
  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> rspack_error::Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}
