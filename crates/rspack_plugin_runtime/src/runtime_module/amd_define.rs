use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
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
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function () {{ throw new Error('define cannot be used indirect'); }}",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::AMD_DEFINE),
    ))
  }
}
