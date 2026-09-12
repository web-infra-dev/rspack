use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  RuntimeTemplateRenderMode, impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct EnsureChunkRuntimeModule {
  has_async_chunks: bool,
}

impl EnsureChunkRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, has_async_chunks: bool) -> Self {
    Self::with_default(runtime_template, has_async_chunks)
  }
}

enum TemplateId {
  Raw,
  WithInline,
  WithInlineArray,
}

impl EnsureChunkRuntimeModule {
  fn template_id(&self, id: TemplateId) -> String {
    match id {
      TemplateId::Raw => self.id().to_string(),
      TemplateId::WithInline => format!("{}_inline", self.id()),
      TemplateId::WithInlineArray => format!("{}_inline_array", self.id()),
    }
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EnsureChunkRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
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
      (
        self.template_id(TemplateId::WithInlineArray),
        include_str!("runtime/ensure_chunk_with_inline_array.ejs").to_string(),
      ),
    ]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let chunk_ukey = self.chunk().expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    let source = if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
      let fetch_priority = if runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY) {
        ", fetchPriority"
      } else {
        ""
      };

      runtime_template.render(
        &self.template_id(TemplateId::Raw),
        Some(serde_json::json!({
          "_fetch_priority": fetch_priority,
          "_chunk_array": render_ensure_chunk_array(context, runtime_requirements),
        })),
      )?
    } else if runtime_requirements.contains(RuntimeGlobals::HAS_CHUNK_ARRAY) {
      runtime_template.render(
        &self.template_id(TemplateId::WithInlineArray),
        Some(serde_json::json!({
          "_chunk_array": render_ensure_chunk_array(context, runtime_requirements),
        })),
      )?
    } else {
      runtime_template.render(&self.template_id(TemplateId::WithInline), None)?
    };

    Ok(source)
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let mut dependencies = RuntimeGlobals::default();
    let mut define = RuntimeGlobals::ENSURE_CHUNK;
    if let Some(chunk_ukey) = self.chunk() {
      if self.has_async_chunks
        || get_chunk_runtime_requirements(compilation, &chunk_ukey)
          .contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      {
        dependencies.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
        define.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
      }
    } else if self.has_async_chunks {
      dependencies.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
      define.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    }
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      define,
      force_context: if compilation.runtime_template.render_mode()
        == RuntimeTemplateRenderMode::Rspack
        && self.chunk().is_some_and(|chunk| {
          get_chunk_runtime_requirements(compilation, &chunk)
            .contains(RuntimeGlobals::HAS_CHUNK_ARRAY)
        }) {
        RuntimeGlobals::ENSURE_CHUNK
      } else {
        RuntimeGlobals::default()
      },
      ..Default::default()
    }
  }
}

/// Shared by the regular and modern-module loaders. Keep per-chunk Promise.all results
/// nested, and look up the current loader for each ID so wrappers and receivers still work.
pub fn render_ensure_chunk_array(
  context: &RuntimeModuleGenerateContext<'_>,
  requirements: &RuntimeGlobals,
) -> String {
  if !requirements.contains(RuntimeGlobals::HAS_CHUNK_ARRAY) {
    return String::new();
  }
  let ensure_chunk = context
    .compilation
    .runtime_template
    .create_module_code_template()
    .render_runtime_globals_without_adding(&RuntimeGlobals::ENSURE_CHUNK);
  let call = if requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY) {
    format!(
      "fetchPriority === undefined ? {ensure_chunk}(chunkId[i]) : {ensure_chunk}(chunkId[i], fetchPriority)"
    )
  } else {
    format!("{ensure_chunk}(chunkId[i])")
  };
  format!(
    "\tif (Array.isArray(chunkId)) {{\n\t\tvar promises = [];\n\t\tfor (var i = 0; i < chunkId.length; i++) {{\n\t\t\tpromises.push({call});\n\t\t}}\n\t\treturn Promise.all(promises);\n\t}}\n"
  )
}
