use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
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
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{}.federation = {{ instance: undefined,bundlerRuntime: undefined }};",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    ))
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
