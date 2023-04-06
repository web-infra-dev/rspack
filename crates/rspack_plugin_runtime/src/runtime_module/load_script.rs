use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct LoadScriptRuntimeModule {
  id: Identifier,
}

impl Default for LoadScriptRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/load_script"),
    }
  }
}

impl RuntimeModule for LoadScriptRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/load_script.js").replace(
      "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
      &compilation.options.output.cross_origin_loading.to_string(),
    ))
    .boxed()
  }
}

impl_runtime_module!(LoadScriptRuntimeModule);
