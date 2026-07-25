use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

static TYPESCRIPT_REST_TEMPLATE: &str = include_str!("runtime/typescript_rest.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct TypeScriptRestRuntimeModule {}

impl TypeScriptRestRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for TypeScriptRestRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &rspack_core::Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::TYPESCRIPT_REST },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), TYPESCRIPT_REST_TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id(), None)
  }
}
