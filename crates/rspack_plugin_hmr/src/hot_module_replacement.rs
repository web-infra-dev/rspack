use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};
use rspack_util::test::is_hot_test;

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementRuntimeModule {
  id: Identifier,
}

impl Default for HotModuleReplacementRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/hot_module_replacement"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/hot_module_replacement.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let content = compilation.runtime_template.render(
      self.id.as_str(),
      Some(serde_json::json!({
        "_is_hot_test": is_hot_test(),
      })),
    )?;

    Ok(content)
  }
}
