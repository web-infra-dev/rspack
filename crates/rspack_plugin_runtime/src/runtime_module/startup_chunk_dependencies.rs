use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_plugin_javascript::runtime::stringify_chunks_to_array;
use rustc_hash::FxHashSet as HashSet;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct StartupChunkDependenciesRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
  chunk: Option<ChunkUkey>,
}

impl StartupChunkDependenciesRuntimeModule {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/startup_chunk_dependencies"),
      async_chunk_loading,
      chunk: None,
    }
  }
}

impl RuntimeModule for StartupChunkDependenciesRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = self.chunk {
      let chunk_ids = compilation
        .chunk_graph
        .get_chunk_entry_dependent_chunks_iterable(
          &chunk_ukey,
          &compilation.chunk_by_ukey,
          &compilation.chunk_group_by_ukey,
        )
        .map(|chunk_ukey| {
          let chunk = compilation
            .chunk_by_ukey
            .get(&chunk_ukey)
            .expect("Chunk not found");
          chunk.expect_id().to_string()
        })
        .collect::<HashSet<_>>();
      let source = if self.async_chunk_loading {
        include_str!("runtime/startup_chunk_dependencies_with_async.js")
      } else {
        include_str!("runtime/startup_chunk_dependencies.js")
      };
      RawSource::from(source.replace("$ChunkIds$", &stringify_chunks_to_array(&chunk_ids))).boxed()
    } else {
      unreachable!("should have chunk for StartupChunkDependenciesRuntimeModule")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(StartupChunkDependenciesRuntimeModule);
