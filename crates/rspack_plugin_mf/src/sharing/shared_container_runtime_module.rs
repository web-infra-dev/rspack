use rspack_core::{
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate, impl_runtime_module
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ShareContainerRuntimeModule {}

impl ShareContainerRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, "share_container_federation")
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ShareContainerRuntimeModule {
  async fn generate(&self, _context: &RuntimeModuleGenerateContext<'_>) -> rspack_error::Result<String> {
    Ok(
      "__webpack_require__.federation = { instance: undefined,bundlerRuntime: undefined };"
        .to_string(),
    )
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
