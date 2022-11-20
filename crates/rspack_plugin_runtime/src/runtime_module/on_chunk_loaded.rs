use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct OnChunkLoadedRuntimeModule {}

impl RuntimeModule for OnChunkLoadedRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/on_chunk_loaded"
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/on_chunk_loaded.js").to_string()).boxed()
  }
}
