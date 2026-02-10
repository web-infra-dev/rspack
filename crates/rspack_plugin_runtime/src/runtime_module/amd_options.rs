use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AmdOptionsRuntimeModule {
  options: String,
}

impl AmdOptionsRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, options: String) -> Self {
    Self::with_default(runtime_template, options)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AmdOptionsRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = {}",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::AMD_OPTIONS),
      self.options,
    ))
  }
}
