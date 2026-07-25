use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

static TYPESCRIPT_AWAITER_TEMPLATE: &str = include_str!("runtime/typescript_awaiter.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct TypeScriptAwaiterRuntimeModule {}

impl TypeScriptAwaiterRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for TypeScriptAwaiterRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &rspack_core::Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::TYPESCRIPT_AWAITER },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      TYPESCRIPT_AWAITER_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id(), None)
  }
}
