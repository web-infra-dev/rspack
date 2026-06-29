use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

static COMPAT_GET_DEFAULT_EXPORT_TEMPLATE: &str =
  include_str!("runtime/compat_get_default_export.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct CompatGetDefaultExportRuntimeModule {}

impl CompatGetDefaultExportRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CompatGetDefaultExportRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      write: { RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      COMPAT_GET_DEFAULT_EXPORT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(&self.id, None)?;

    Ok(source)
  }
}
