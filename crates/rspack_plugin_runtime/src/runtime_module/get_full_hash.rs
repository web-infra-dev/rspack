use rspack_core::{
  RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GetFullHashRuntimeModule {}

impl GetFullHashRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GetFullHashRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_full_hash.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let compilation = context.compilation;
    let source = context.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_hash": format!("\"{}\"", compilation.get_hash().unwrap_or("XXXX"))
      })),
    )?;

    Ok(source)
  }

  fn full_hash(&self) -> bool {
    true
  }
}
