use cow_utils::CowUtils;
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

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawStringSource::from(
        include_str!("runtime/get_unique_id.js")
          .cow_replace("$BUNDLER_NAME$", &self.bundler_name)
          .cow_replace("$BUNDLER_VERSION$", &self.bundler_version)
          .into_owned(),
      )
      .boxed(),
    )
  }
}
