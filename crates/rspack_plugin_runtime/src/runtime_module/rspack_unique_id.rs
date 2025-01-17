use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule, RuntimeModuleStage,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RspackUniqueIdRuntimeModule {
  id: Identifier,
  bundler_name: String,
  bundler_version: String,
}

impl RspackUniqueIdRuntimeModule {
  pub fn new(bundler_name: String, bundler_version: String) -> Self {
    Self::with_default(
      Identifier::from("webpack/runtime/rspack_unique_id"),
      bundler_name,
      bundler_version,
    )
  }
}

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

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let source = compilation.runtime_template.render(
      &self.id,
      Some(serde_json::json!({
        "BUNDLER_NAME": &self.bundler_name,
        "BUNDLER_VERSION":&self.bundler_version,
      })),
    )?;

    Ok(RawStringSource::from(source).boxed())
  }
}
