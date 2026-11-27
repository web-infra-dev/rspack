use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
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

#[async_trait::async_trait]
impl RuntimeModule for EnsureChunkRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/ensure_chunk.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let chunk_ukey = self.chunk().expect("should have chunk");
    let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_with_handlers": runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS),
        "_with_fetch_priority": runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY),
        "_with_hmr": runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS),
        "_hot": compilation.hot,
      })),
    )
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let mut dependencies = RuntimeGlobals::default();
    let mut define = RuntimeGlobals::ENSURE_CHUNK;
    let mut force_context = RuntimeGlobals::default();
    if compilation.hot {
      dependencies.insert(RuntimeGlobals::REQUIRE_SCOPE);
      force_context = RuntimeGlobals::ENSURE_CHUNK;
    }
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
      force_context,
      ..Default::default()
    }
  }
}
