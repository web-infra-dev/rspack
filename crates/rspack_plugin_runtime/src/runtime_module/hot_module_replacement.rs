use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct HotModuleReplacementRuntimeModule {}

impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/hot_module_replacement".to_string()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/hot_module_replacement.js").to_string()).boxed()
  }
}
