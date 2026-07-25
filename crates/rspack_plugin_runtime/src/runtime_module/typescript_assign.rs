use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

static TYPESCRIPT_ASSIGN_TEMPLATE: &str = include_str!("runtime/typescript_assign.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct TypeScriptAssignRuntimeModule {}

impl TypeScriptAssignRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for TypeScriptAssignRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::HAS_OWN_PROPERTY,
      define: { RuntimeGlobals::TYPESCRIPT_ASSIGN },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      TYPESCRIPT_ASSIGN_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id(), None)
  }
}
