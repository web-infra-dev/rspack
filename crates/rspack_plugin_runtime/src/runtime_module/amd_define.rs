use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AmdDefineRuntimeModule {}

impl AmdDefineRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AmdDefineRuntimeModule {
  fn additional_write_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    RuntimeGlobals::AMD_DEFINE
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function () {{ throw new Error('define cannot be used indirect'); }}",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::AMD_DEFINE),
    ))
  }
}
