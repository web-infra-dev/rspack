use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct StartupEntrypointRuntimeModule {
  id: Identifier,
  async_chunk_loading: bool,
}

impl StartupEntrypointRuntimeModule {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/start_entry_point"),
      async_chunk_loading,
    }
  }
}

impl RuntimeModule for StartupEntrypointRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    let source = if self.async_chunk_loading {
      include_str!("runtime/start_entry_point_with_async.js")
    } else {
      include_str!("runtime/start_entry_point.js")
    };
    RawSource::from(source).boxed()
  }
}

impl_runtime_module!(StartupEntrypointRuntimeModule);
