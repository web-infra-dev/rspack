use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct OnChunkLoadedRuntimeModule {
  id: Identifier,
}

impl Default for OnChunkLoadedRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/on_chunk_loaded"),
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for OnChunkLoadedRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/on_chunk_loaded.js")).boxed())
  }
}
