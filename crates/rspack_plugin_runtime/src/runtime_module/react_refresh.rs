use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct ReactRefreshRuntimeModule {
  id: Identifier,
}

impl Default for ReactRefreshRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/react_refresh"),
    }
  }
}

impl RuntimeModule for ReactRefreshRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/react_refresh.js")).boxed()
  }
}

impl_runtime_module!(ReactRefreshRuntimeModule);
