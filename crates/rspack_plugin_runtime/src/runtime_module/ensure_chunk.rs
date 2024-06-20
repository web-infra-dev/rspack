use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};
use rspack_identifier::Identifier;

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct EnsureChunkRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for EnsureChunkRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/ensure_chunk"), None)
  }
}

impl RuntimeModule for EnsureChunkRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self.chunk.expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    Ok(
      RawSource::from(
        match runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
          true => include_str!("runtime/ensure_chunk.js"),
          false => include_str!("runtime/ensure_chunk_with_inline.js"),
        },
      )
      .boxed(),
    )
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
