use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleRuntimeRequirements,
  RuntimeTemplate, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_globals_from_ejs;

static HOT_CONTEXT_TEMPLATE: &str = include_str!("runtime/hot_context.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotContextRuntimeModule {}

impl HotContextRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotContextRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), HOT_CONTEXT_TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id().as_str(), None)
  }

  fn runtime_requirements(&self, _compilation: &Compilation) -> RuntimeModuleRuntimeRequirements {
    extract_runtime_globals_from_ejs(HOT_CONTEXT_TEMPLATE)
  }
}
