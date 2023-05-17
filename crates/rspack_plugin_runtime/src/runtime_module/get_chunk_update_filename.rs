use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, PathData, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

// TODO workaround for get_chunk_update_filename
#[derive(Debug, Eq)]
pub struct GetChunkUpdateFilenameRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for GetChunkUpdateFilenameRuntimeModule {
  fn default() -> Self {
    Self {
      chunk: None,
      id: Identifier::from("webpack/runtime/get_chunk_update_filename"),
    }
  }
}

impl RuntimeModule for GetChunkUpdateFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
  fn generate(&self, compilation: &Compilation) -> BoxSource {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation
        .chunk_by_ukey
        .get(&chunk_ukey)
        .expect("Chunk not found");
      let filename = compilation.get_path(
        &compilation.options.output.hot_update_chunk_filename,
        PathData::default()
          .chunk(chunk)
          .hash(format!("' + {}() + '", RuntimeGlobals::GET_FULL_HASH).as_str())
          .id("' + chunkId + '")
          .runtime(&chunk.runtime),
      );
      RawSource::from(format!(
        "{} = function (chunkId) {{
            return '{}';
         }};
        ",
        RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME,
        filename
      ))
      .boxed()
    } else {
      unreachable!("should attach chunk for get_main_filename")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(GetChunkUpdateFilenameRuntimeModule);
