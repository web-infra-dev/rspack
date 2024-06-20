use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct MakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for MakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/make_namespace_object"))
  }
}

impl RuntimeModule for MakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/make_namespace_object.js")).boxed())
  }
}
