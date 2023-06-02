use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct MakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for MakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/make_namespace_object"),
    }
  }
}

impl RuntimeModule for MakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/make_namespace_object.js")).boxed()
  }
}

impl_runtime_module!(MakeNamespaceObjectRuntimeModule);
