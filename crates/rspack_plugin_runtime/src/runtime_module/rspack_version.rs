use rspack_core::{
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
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
