use rspack_core::{
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RspackUniqueIdRuntimeModule {
  bundler_name: String,
  bundler_version: String,
}

impl RspackUniqueIdRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    bundler_name: String,
    bundler_version: String,
  ) -> Self {
    Self::with_default(runtime_template, bundler_name, bundler_version)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RspackUniqueIdRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_unique_id.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let source = context.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_bundler_name": &self.bundler_name,
        "_bundler_version": &self.bundler_version,
      })),
    )?;

    Ok(source)
  }
}
