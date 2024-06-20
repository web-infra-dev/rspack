use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct RspackUniqueIdRuntimeModule {
  id: Identifier,
  bundler_name: String,
}

impl RspackUniqueIdRuntimeModule {
  pub fn new(bundler_name: String) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/rspack_unique_id"),
      bundler_name,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
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
      RawSource::from(
        include_str!("runtime/get_unique_id.js")
          .replace("$BUNDLER_NAME$", &self.bundler_name)
          .replace(
            "$RUNTIME_GET_VERSION$",
            RuntimeGlobals::RSPACK_VERSION.to_string().as_ref(),
          ),
      )
      .boxed(),
    )
  }
}
