use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateScriptUrlRuntimeModule {
  id: Identifier,
}

impl Default for CreateScriptUrlRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/create_script_url"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateScriptUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/create_script_url.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_trusted_types": compilation.options.output.trusted_types.is_some(),
      })),
    )?;

    Ok(source)
  }
}
