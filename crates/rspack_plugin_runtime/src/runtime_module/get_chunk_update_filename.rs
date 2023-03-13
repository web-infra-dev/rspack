use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

use crate::impl_runtime_module;

// TODO workaround for get_chunk_update_filename
#[derive(Debug, Default, Eq)]
pub struct GetChunkUpdateFilenameRuntimeModule {}

impl RuntimeModule for GetChunkUpdateFilenameRuntimeModule {
  fn name(&self) -> String {
    "webpack/runtime/get_chunk_update_filename".to_owned()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/get_chunk_update_filename.js")).boxed()
  }
}

impl_runtime_module!(GetChunkUpdateFilenameRuntimeModule);
