use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct OnChunkLoadedRuntimeModule {}

impl RuntimeModule for OnChunkLoadedRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/on_chunk_loaded".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/on_chunk_loaded.js")).boxed()
  }
}

impl_runtime_module!(OnChunkLoadedRuntimeModule);
