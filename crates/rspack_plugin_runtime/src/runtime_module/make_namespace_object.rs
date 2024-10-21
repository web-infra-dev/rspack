use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl MakeNamespaceObjectRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/make_namespace_object.js").to_string())
  }
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
}
