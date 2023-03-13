use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct GlobalRuntimeModule {}

impl RuntimeModule for GlobalRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/global".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/global.js")).boxed()
  }
}

impl_runtime_module!(GlobalRuntimeModule);
