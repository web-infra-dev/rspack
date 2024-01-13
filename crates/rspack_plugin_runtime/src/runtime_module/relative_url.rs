use rspack_common::SourceMapKind;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct RelativeUrlRuntimeModule {
  id: Identifier,
}

impl Default for RelativeUrlRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/relative_url"),
      source_map_kind: SourceMapKind::None,
    }
  }
}

impl RuntimeModule for RelativeUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _: &Compilation) -> BoxSource {
    RawSource::from(include_str!("runtime/relative_url.js")).boxed()
  }
}
