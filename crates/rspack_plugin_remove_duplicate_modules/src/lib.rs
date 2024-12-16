use std::sync::Arc;

use rspack_core::{
  incremental::Mutation, ChunkUkey, Compilation, CompilationOptimizeChunks, ModuleIdentifier,
  Plugin,
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
    let new_chunk_ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
    if let Some(mutations) = compilation.incremental.mutations_write() {
      mutations.add(Mutation::ChunkAdd {
        chunk: new_chunk_ukey,
      });
    }
    let new_chunk = compilation.chunk_by_ukey.expect_get_mut(&new_chunk_ukey);
    *new_chunk.chunk_reason_mut() = Some("modules are shared across multiple chunks".into());
    compilation.chunk_graph.add_chunk(new_chunk_ukey);

    for chunk_ukey in &chunks {
      let [Some(new_chunk), Some(origin)] = compilation
        .chunk_by_ukey
        .get_many_mut([&new_chunk_ukey, chunk_ukey])
      else {
        panic!("should have both chunks")
      };
      origin.split(new_chunk, &mut compilation.chunk_group_by_ukey);
      if let Some(mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkSplit {
          from: *chunk_ukey,
          to: new_chunk_ukey,
        });
      }
    }

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
