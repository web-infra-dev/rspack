use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, RuntimeVariable, impl_runtime_module,
};

use crate::{get_chunk_runtime_requirements, is_modern_module_library_chunk};

static MAKE_DEFERRED_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/make_deferred_namespace_object.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeDeferredNamespaceObjectRuntimeModule {
  chunk_ukey: ChunkUkey,
}

impl MakeDeferredNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, chunk_ukey: ChunkUkey) -> Self {
    Self::with_default(runtime_template, chunk_ukey)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeDeferredNamespaceObjectRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      MAKE_DEFERRED_NAMESPACE_OBJECT_TEMPLATE.to_string(),
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
    let uses_direct_initializers = is_modern_module_library_chunk(&self.chunk_ukey, compilation);
    let source = runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_module_cache": runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
        "_has_async": has_async,
        "_uses_direct_initializers": uses_direct_initializers,
      })),
    )?;

    Ok(source)
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    let uses_direct_initializers = is_modern_module_library_chunk(&self.chunk_ukey, compilation);
    let mut dependencies = RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT;
    if !uses_direct_initializers {
      dependencies.insert(RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_CACHE);
    }
    if !uses_direct_initializers
      && get_chunk_runtime_requirements(compilation, &self.chunk_ukey)
        .contains(RuntimeGlobals::ASYNC_MODULE)
    {
      dependencies.insert(RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL);
    }
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies,
      define: { RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT },
      ..Default::default()
    }
  }
}
