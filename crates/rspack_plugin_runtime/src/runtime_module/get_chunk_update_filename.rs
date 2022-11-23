use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};

// TODO workaround for get_chunk_update_filename
#[derive(Debug, Default)]
pub struct GetChunkUpdateFilenameRuntimeModule {}

impl RuntimeModule for GetChunkUpdateFilenameRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/get_chunk_update_filename".to_string()
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/get_chunk_update_filename.js").to_string()).boxed()
  }
}
