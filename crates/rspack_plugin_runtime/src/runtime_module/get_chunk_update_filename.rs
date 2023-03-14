use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

// TODO workaround for get_chunk_update_filename
#[derive(Debug, Eq)]
pub struct GetChunkUpdateFilenameRuntimeModule {
  id: Identifier,
}

impl Default for GetChunkUpdateFilenameRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/get_chunk_update_filename"),
    }
  }
}

impl RuntimeModule for GetChunkUpdateFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/get_chunk_update_filename.js")).boxed()
  }
}

impl_runtime_module!(GetChunkUpdateFilenameRuntimeModule);
