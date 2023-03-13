use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct HasOwnPropertyRuntimeModule {}

impl RuntimeModule for HasOwnPropertyRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/has_own_property".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/has_own_property.js")).boxed()
  }
}

impl_runtime_module!(HasOwnPropertyRuntimeModule);
