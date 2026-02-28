use rspack_core::{
  RuntimeCodeTemplate, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub(crate) struct EsmRegisterModuleRuntimeModule {}

impl EsmRegisterModuleRuntimeModule {
  pub(crate) fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
  pub(crate) fn runtime_id(runtime_template: &RuntimeCodeTemplate) -> String {
    format!(
      "{}.add",
      runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE)
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for EsmRegisterModuleRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    Ok(format!(
      "{} = function registerModules(modules) {{ Object.assign({}, modules) }}\n",
      Self::runtime_id(context.runtime_template),
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::MODULE_FACTORIES),
    ))
  }
}
