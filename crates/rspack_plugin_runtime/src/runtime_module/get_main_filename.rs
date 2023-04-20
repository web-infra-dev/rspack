use rspack_core::{
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeModule, RuntimeSpec,
};
use rspack_identifier::Identifier;

use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct GetMainFilenameRuntimeModule {
  chunk: Option<ChunkUkey>,
  id: Identifier,
}

impl Default for GetMainFilenameRuntimeModule {
  fn default() -> Self {
    Self {
      chunk: None,
      id: Identifier::from("webpack/runtime/get_main_filename"),
    }
  }
}

impl RuntimeModule for GetMainFilenameRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
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

impl_runtime_module!(GetMainFilenameRuntimeModule);

#[inline]
fn stringify_runtime(runtime: &RuntimeSpec) -> String {
  Vec::from_iter(runtime.iter().map(|s| (*s).as_ref())).join("_")
}
