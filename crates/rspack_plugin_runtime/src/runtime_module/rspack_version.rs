use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct RspackVersionRuntimeModule {
  id: Identifier,
  version: String,
}

impl RspackVersionRuntimeModule {
  pub fn new(version: String) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/rspack_version"),
      version,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for RspackVersionRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(
      RawSource::from(
        include_str!("runtime/get_version.js")
          .replace("$VERSION$", &self.version)
          .replace(
            "$RUNTIME_EXPOSE_GLOBAL$",
            RuntimeGlobals::RSPACK_EXPOSE_GLOBAL.to_string().as_str(),
          ),
      )
      .boxed(),
    )
  }
}
