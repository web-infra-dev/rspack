use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct HotModuleReplacementRuntimeModule {}

impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/hot_module_replacement".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/hot_module_replacement.js")).boxed()
  }
}

impl_runtime_module!(HotModuleReplacementRuntimeModule);
