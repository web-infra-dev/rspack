use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleStage, RuntimeTemplate, impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RspackUniqueIdRuntimeModule {
  id: Identifier,
  bundler_name: String,
  bundler_version: String,
}

impl RspackUniqueIdRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    bundler_name: String,
    bundler_version: String,
  ) -> Self {
    Self::with_default(
      Identifier::from(format!(
        "{}rspack_unique_id",
        runtime_template.runtime_module_prefix()
      )),
      bundler_name,
      bundler_version,
    )
  }
}

#[async_trait::async_trait]
impl RuntimeModule for RspackUniqueIdRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
  fn name(&self) -> Identifier {
    self.id
  }
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      include_str!("runtime/get_unique_id.ejs").to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "_bundler_name": &self.bundler_name,
        "_bundler_version": &self.bundler_version,
      })),
    )?;

    Ok(source)
  }
}
