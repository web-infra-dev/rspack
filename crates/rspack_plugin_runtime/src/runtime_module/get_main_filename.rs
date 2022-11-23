use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct GetMainFilenameRuntimeModule {
  chunk: Option<ChunkUkey>,
}

impl RuntimeModule for GetMainFilenameRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/get_main_filename".to_string()
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation
        .chunk_by_ukey
        .get(&chunk_ukey)
        .expect("Chunk not found");
      RawSource::from(
        include_str!("runtime/get_update_manifest_filename.js")
          .to_string()
          .replace("$CHUNK_ID$", &chunk.id),
      )
      .boxed()
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
