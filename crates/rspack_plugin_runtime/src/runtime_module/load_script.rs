use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct LoadScriptRuntimeModule {}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn identifier(&self) -> &str {
    "webpack/runtime/load_script"
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/load_script.js").to_string()).boxed()
  }

  fn attach(&mut self, _chunk: ChunkUkey) {}
}
