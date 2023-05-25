use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for CreateFakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/create_fake_namespace_object"),
    }
  }
}

impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/create_fake_namespace_object.js")).boxed()
  }
}

impl_runtime_module!(CreateFakeNamespaceObjectRuntimeModule);
