use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct GlobalRuntimeModule {}

impl RuntimeModule for GlobalRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/global".to_string()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/global.js").to_string()).boxed()
  }
}
