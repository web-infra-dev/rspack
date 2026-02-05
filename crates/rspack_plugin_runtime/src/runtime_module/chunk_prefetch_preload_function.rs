use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ChunkPrefetchPreloadFunctionRuntimeModule {
  runtime_function: RuntimeGlobals,
  runtime_handlers: RuntimeGlobals,
}

impl ChunkPrefetchPreloadFunctionRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    child_type: &str,
    runtime_function: RuntimeGlobals,
    runtime_handlers: RuntimeGlobals,
  ) -> Self {
    Self::with_name(
      runtime_template,
      child_type,
      runtime_function,
      runtime_handlers,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ChunkPrefetchPreloadFunctionRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/chunk_prefetch_preload_function.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_runtime_handlers":  compilation.runtime_template.render_runtime_globals(&self.runtime_handlers),
        "_runtime_function": compilation.runtime_template.render_runtime_globals(&self.runtime_function),
      })),
    )?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    self.runtime_handlers
  }
}
