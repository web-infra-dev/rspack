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

enum TemplateId {
  Raw,
  WithInline,
}

impl EnsureChunkRuntimeModule {
  fn template_id(&self, id: TemplateId) -> String {
    match id {
      TemplateId::Raw => self.id.to_string(),
      TemplateId::WithInline => format!("{}_inline", &self.id),
    }
  }
}

impl RuntimeModule for EnsureChunkRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![
      (
        self.template_id(TemplateId::Raw),
        include_str!("runtime/ensure_chunk.ejs").to_string(),
      ),
      (
        self.template_id(TemplateId::WithInline),
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
        &self.template_id(TemplateId::Raw),
        Some(serde_json::json!({
          "_args": format!("chunkId{}", fetch_priority),
          "_fetch_priority": fetch_priority,
        })),
      )?
    } else {
      compilation
        .runtime_template
        .render(&self.template_id(TemplateId::WithInline), None)?
    };

    Ok(RawStringSource::from(source).boxed())
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}
