use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, RuntimeVariable,
  impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  id: Identifier,
  chunk_ukey: ChunkUkey,
}

impl MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}make_optimized_deferred_namespace_object",
        runtime_template.runtime_module_prefix()
      )),
      chunk_ukey,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/make_optimized_deferred_namespace_object.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let has_async = get_chunk_runtime_requirements(compilation, &self.chunk_ukey)
      .contains(RuntimeGlobals::ASYNC_MODULE);
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_module_cache": compilation.runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
        "_has_async": has_async,
      })),
    )?;

    Ok(source)
  }
}
