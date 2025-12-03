use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, RuntimeVariable, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncRuntimeModule {
  id: Identifier,
}
impl Default for AsyncRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/async_module"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncRuntimeModule {
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_module_cache": compilation.runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
      })),
    )
  }

  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/async_module.ejs").to_string(),
    )]
  }
}
