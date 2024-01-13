use rspack_common::SourceMapKind;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct PublicPathRuntimeModule {
  id: Identifier,
  public_path: Box<str>,
}

impl PublicPathRuntimeModule {
  pub fn new(public_path: Box<str>) -> Self {
    Self {
      id: Identifier::from("webpack/runtime/public_path"),
      public_path,
      source_map_kind: SourceMapKind::None,
    }
  }
}

impl RuntimeModule for PublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> BoxSource {
    RawSource::from(
      include_str!("runtime/public_path.js")
        .replace("__PUBLIC_PATH_PLACEHOLDER__", &self.public_path),
    )
    .boxed()
  }
}
