use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct HasOwnPropertyRuntimeModule {}

impl RuntimeModule for HasOwnPropertyRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/has_own_property".to_string()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/has_own_property.js").to_string()).boxed()
  }
}
