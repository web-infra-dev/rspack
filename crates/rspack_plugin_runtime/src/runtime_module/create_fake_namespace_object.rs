use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for CreateFakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/create_fake_namespace_object"),
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/create_fake_namespace_object.js")).boxed())
  }
}
