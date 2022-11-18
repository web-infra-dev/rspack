use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct CssLoadingRuntimeModule {}

impl RuntimeModule for CssLoadingRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/css_loading"
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/css_loading.js").to_string()).boxed()
  }

  fn attach(&mut self, _chunk: ChunkUkey) {}
}
