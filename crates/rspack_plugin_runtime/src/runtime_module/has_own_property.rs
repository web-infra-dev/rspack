use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeModule, RuntimeTemplate, impl_runtime_module};

#[impl_runtime_module]
#[derive(Debug)]
pub struct HasOwnPropertyRuntimeModule {
  id: Identifier,
}

impl HasOwnPropertyRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}has_own_property",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HasOwnPropertyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/has_own_property.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }
}
