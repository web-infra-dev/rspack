use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
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
  fn name(&self) -> String {
    "webpack/runtime/ensure_chunk".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(match self.has_ensure_chunk_handlers {
      true => include_str!("runtime/ensure_chunk.js"),
      false => include_str!("runtime/ensure_chunk_with_inline.js"),
    })
    .boxed()
  }
}

impl_runtime_module!(EnsureChunkRuntimeModule);
