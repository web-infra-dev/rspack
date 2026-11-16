use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage,
  RuntimeTemplate, impl_runtime_module,
};

use crate::{
  container::federation_data_runtime_module::render_federation_require_factory,
  utils::federation_require_requirement,
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
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let adapter = render_federation_require_factory(
      context.compilation,
      self.chunk().expect("attached share container runtime"),
      context.runtime_template,
    );
    Ok(format!(
      "{}.federation = {{ instance: undefined,bundlerRuntime: undefined }};{adapter}",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
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
      dependencies: { federation_require_requirement(compilation) },
      ..Default::default()
    }
  }
}
