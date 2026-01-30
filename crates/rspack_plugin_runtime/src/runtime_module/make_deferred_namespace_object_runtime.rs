use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, RuntimeVariable,
  impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeDeferredNamespaceObjectRuntimeModule {
  id: Identifier,
  chunk_ukey: ChunkUkey,
}

impl MakeDeferredNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}make_deferred_namespace_object",
        runtime_template.runtime_module_prefix()
      )),
      chunk_ukey,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeDeferredNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/make_deferred_namespace_object.ejs").to_string(),
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

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT
      | RuntimeGlobals::REQUIRE
      | RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
  }
}
