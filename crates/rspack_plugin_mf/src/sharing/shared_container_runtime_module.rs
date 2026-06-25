use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};

use crate::utils::{runtime_require_scope_name, runtime_require_scope_requirement};

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
      runtime_require_scope_name(context.runtime_template)
    ))
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { runtime_require_scope_requirement(compilation) },
      ..Default::default()
    }
  }
}
