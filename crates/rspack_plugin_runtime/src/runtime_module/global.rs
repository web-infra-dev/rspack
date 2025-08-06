use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct GlobalRuntimeModule {
  id: Identifier,
}

impl Default for GlobalRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/global"))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for GlobalRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/global.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }
}
