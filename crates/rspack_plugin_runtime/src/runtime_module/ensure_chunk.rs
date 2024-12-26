use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
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

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.id.to_string(),
        include_str!("runtime/ensure_chunk.ejs").to_string(),
      ),
      (
        format!("{}-inline", self.id),
        include_str!("runtime/ensure_chunk_with_inline.ejs").to_string(),
      ),
    ]
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk_ukey = self.chunk.expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    let source = if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
      let fetch_priority = if runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY) {
        ", fetchPriority"
      } else {
        ""
      };

      compilation.runtime_template.render(
        &self.id,
        Some(serde_json::json!({
          "ARGS": format!("chunkId{}", fetch_priority),
          "FETCH_PRIORITY": fetch_priority,
        })),
      )?
    } else {
      compilation
        .runtime_template
        .render(&format!("{}-inline", self.id), None)?
    };

    Ok(RawStringSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
