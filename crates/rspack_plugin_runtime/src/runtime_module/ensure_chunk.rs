use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct EnsureChunkRuntimeModule {
  has_ensure_chunk_handlers: bool,
}

impl EnsureChunkRuntimeModule {
  pub fn new(has_ensure_chunk_handlers: bool) -> Self {
    Self {
      has_ensure_chunk_handlers,
    }
  }
}

impl RuntimeModule for EnsureChunkRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/ensure_chunk"
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(match self.has_ensure_chunk_handlers {
      true => include_str!("runtime/ensure_chunk.js").to_string(),
      false => include_str!("runtime/ensure_chunk_with_inline.js").to_string(),
    })
    .boxed()
  }

  fn attach(&mut self, _chunk: ChunkUkey) {}
}
