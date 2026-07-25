use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

static DEFINE_ESM_MARKER_TEMPLATE: &str = include_str!("runtime/define_esm_marker.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefineEsmMarkerRuntimeModule {}

impl DefineEsmMarkerRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for DefineEsmMarkerRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &rspack_core::Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::DEFINE_ESM_MARKER },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      DEFINE_ESM_MARKER_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id(), None)
  }
}
