use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

static DEFINE_ESM_EXPORTS_TEMPLATE: &str = include_str!("runtime/define_esm_exports.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefineEsmExportsRuntimeModule {}

impl DefineEsmExportsRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for DefineEsmExportsRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::MAKE_NAMESPACE_OBJECT | RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      define: { RuntimeGlobals::DEFINE_ESM_EXPORTS },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      DEFINE_ESM_EXPORTS_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(self.id(), None)?;

    Ok(source)
  }
}
