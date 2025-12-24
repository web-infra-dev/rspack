use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, RuntimeTemplate, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ToBinaryRuntimeModule {
  id: Identifier,
}

impl ToBinaryRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}to_binary",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ToBinaryRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/to_binary.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let is_node_platform = compilation.platform.is_node();
    let is_web_platform = compilation.platform.is_web();
    let is_neutral_platform = compilation.platform.is_neutral();

    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_is_node_platform": is_node_platform,
        "_is_web_platform": is_web_platform,
        "_is_neutral_platform": is_neutral_platform,
      })),
    )?;

    Ok(source)
  }
}
