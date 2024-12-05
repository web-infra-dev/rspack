use rspack_collections::Identifier;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  Compilation, RuntimeModule,
};

#[impl_runtime_module]
#[derive(Debug)]
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
    Ok(RawStringSource::from_static(include_str!("runtime/make_namespace_object.js")).boxed())
  }
}
