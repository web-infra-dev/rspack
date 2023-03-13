use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct NormalRuntimeModule {
  pub identifier: &'static str,
  pub sources: &'static str,
}

impl NormalRuntimeModule {
  pub fn new(identifier: &'static str, sources: &'static str) -> Self {
    Self {
      identifier,
      sources,
    }
  }
}

impl RuntimeModule for NormalRuntimeModule {
  fn name(&self) -> String {
    self.identifier.to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(self.sources).boxed()
  }
}

impl_runtime_module!(NormalRuntimeModule);
