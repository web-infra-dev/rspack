use cow_utils::CowUtils;
use rspack_collections::{Identifiable, Identifier};
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, OriginalSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
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

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self.chunk.expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    let generated_code = match runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    {
      true => include_str!("runtime/ensure_chunk.js")
        .cow_replace(
          "$FETCH_PRIORITY$",
          if runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY) {
            ", fetchPriority"
          } else {
            ""
          },
        )
        .into_owned(),
      false => include_str!("runtime/ensure_chunk_with_inline.js").to_string(),
    };

    let source = if self.source_map_kind.enabled() {
      OriginalSource::new(generated_code, self.identifier().to_string()).boxed()
    } else {
      RawSource::from(generated_code).boxed()
    };
    Ok(source)
  }
}
