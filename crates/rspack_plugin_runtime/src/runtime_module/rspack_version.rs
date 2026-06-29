use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RspackVersionRuntimeModule {
  version: String,
}

impl RspackVersionRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, version: String) -> Self {
    Self::with_default(runtime_template, version)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RspackVersionRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      write: { RuntimeGlobals::RSPACK_VERSION },
      force_context: RuntimeGlobals::RSPACK_VERSION,
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_version.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_version": format!("\"{}\"", &self.version),
      })),
    )?;

    Ok(source)
  }
}
