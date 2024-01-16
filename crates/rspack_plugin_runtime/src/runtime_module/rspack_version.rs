use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

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
    }
  }
}

impl RuntimeModule for RspackVersionRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/get_version.js").replace("$VERSION$", &self.version))
      .boxed()
  }
}

impl_runtime_module!(RspackVersionRuntimeModule);
