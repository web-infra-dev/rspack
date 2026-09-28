use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};
#[cfg(allocative)]
use rspack_util::allocative;

#[impl_runtime_module]
#[derive(Debug)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct NonceRuntimeModule {}

impl NonceRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for NonceRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::SCRIPT_NONCE },
      force_context: RuntimeGlobals::SCRIPT_NONCE,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = undefined;",
      context
        .runtime_template
        .render_runtime_global_definition(&RuntimeGlobals::SCRIPT_NONCE)
    ))
  }
}
