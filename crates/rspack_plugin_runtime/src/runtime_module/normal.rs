use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct NormalRuntimeModule {
  pub identifier: Identifier,
  pub sources: &'static str,
}

impl NormalRuntimeModule {
  pub fn new(identifier: &'static str, sources: &'static str) -> Self {
    Self {
      identifier: Identifier::from(identifier),
      sources,
    }
  }
}

impl RuntimeModule for NormalRuntimeModule {
  fn name(&self) -> Identifier {
    self.identifier
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(self.sources).boxed()
  }
}

impl_runtime_module!(NormalRuntimeModule);
