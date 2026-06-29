use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, RuntimeVariable, impl_runtime_module,
};

use crate::get_chunk_runtime_requirements;

static MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/make_optimized_deferred_namespace_object.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  chunk_ukey: ChunkUkey,
}

impl MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(runtime_template, chunk_ukey)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeOptimizedDeferredNamespaceObjectRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let runtime_template = context.runtime_template;
    let has_async = get_chunk_runtime_requirements(compilation, &self.chunk_ukey)
      .contains(RuntimeGlobals::ASYNC_MODULE);
    let source = runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_module_cache": runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
        "_has_async": has_async,
      })),
    )?;

    Ok(source)
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let mut dependencies = RuntimeGlobals::REQUIRE;
    if get_chunk_runtime_requirements(compilation, &self.chunk_ukey)
      .contains(RuntimeGlobals::ASYNC_MODULE)
    {
      dependencies.insert(
        RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
          | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL,
      );
    }
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      write: { RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT },
      ..Default::default()
    }
  }
}
