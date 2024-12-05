use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct OnChunkLoadedRuntimeModule {
  id: Identifier,
}

impl Default for OnChunkLoadedRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/on_chunk_loaded"))
  }
}

impl RuntimeModule for OnChunkLoadedRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawStringSource::from_static(include_str!("runtime/on_chunk_loaded.js")).boxed())
  }
}
