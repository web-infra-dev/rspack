use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Default, Eq)]
pub struct LoadScriptRuntimeModule {}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/load_script".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/load_script.js")).boxed()
  }
}

impl_runtime_module!(LoadScriptRuntimeModule);
