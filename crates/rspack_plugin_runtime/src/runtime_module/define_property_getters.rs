use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct DefinePropertyGettersRuntimeModule {
  id: Identifier,
}

impl Default for DefinePropertyGettersRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/define_property_getters"),
    }
  }
}

impl RuntimeModule for DefinePropertyGettersRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/define_property_getters.js")).boxed()
  }
}

impl_runtime_module!(DefinePropertyGettersRuntimeModule);
