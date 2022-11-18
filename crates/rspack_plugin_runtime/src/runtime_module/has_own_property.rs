use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct HasOwnPropertyRuntimeModule {}

impl RuntimeModule for HasOwnPropertyRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/has_own_property"
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/has_own_property.js").to_string()).boxed()
  }

  fn attach(&mut self, _chunk: ChunkUkey) {}
}
