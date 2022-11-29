use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule, RuntimeSpec,
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
          .replace("$CHUNK_ID$", &stringify_runtime(&chunk.runtime)),
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

#[inline]
fn stringify_runtime(runtime: &RuntimeSpec) -> String {
  Vec::from_iter(runtime.iter().map(|s| s.as_str())).join("_")
}
