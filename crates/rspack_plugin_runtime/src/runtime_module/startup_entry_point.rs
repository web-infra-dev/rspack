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
      id: Identifier::from("webpack/runtime/startup_entrypoint"),
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
      include_str!("runtime/startup_entrypoint_with_async.js")
    } else {
      include_str!("runtime/startup_entrypoint.js")
    };
    RawSource::from(source).boxed()
  }
}

impl_runtime_module!(StartupEntrypointRuntimeModule);
