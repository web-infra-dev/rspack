use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct NodeModuleDecoratorRuntimeModule {
  id: Identifier,
}

impl Default for NodeModuleDecoratorRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/node_module_decorator"),
    }
  }
}

impl RuntimeModule for NodeModuleDecoratorRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/node_module_decorator.js")).boxed()
  }
}

impl_runtime_module!(NodeModuleDecoratorRuntimeModule);
