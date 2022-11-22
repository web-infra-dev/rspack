use crate::runtime_module::utils::{get_initial_chunk_ids, stringify_chunks};
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct CommonJsChunkLoadingRuntimeModule {
  chunk: Option<ChunkUkey>,
}

impl RuntimeModule for CommonJsChunkLoadingRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/common_js_chunk_loading".to_string()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let initial_chunks = get_initial_chunk_ids(self.chunk, compilation);
    RawSource::from(
      include_str!("runtime/common_js_chunk_loading.js")
        .to_string()
        .replace("INSTALLED_CHUNKS", &stringify_chunks(&initial_chunks, 1))
        // TODO
        .replace("JS_MATCHER", "chunkId"),
    )
    .boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
