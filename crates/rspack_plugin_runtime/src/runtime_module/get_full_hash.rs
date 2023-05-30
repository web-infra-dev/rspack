use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct GetFullHashRuntimeModule {
  id: Identifier,
}

impl Default for GetFullHashRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_full_hash"),
    }
  }
}

impl RuntimeModule for GetFullHashRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    RawSource::from(
      include_str!("runtime/get_full_hash.js")
        .replace("$HASH$", compilation.get_hash().unwrap_or("XXXX")),
    )
    .boxed()
  }

  fn cacheable(&self) -> bool {
    false
  }
}

impl_runtime_module!(GetFullHashRuntimeModule);
