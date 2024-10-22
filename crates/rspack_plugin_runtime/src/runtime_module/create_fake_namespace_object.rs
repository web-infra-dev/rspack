use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl Default for CreateFakeNamespaceObjectRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from(
      "webpack/runtime/create_fake_namespace_object",
    ))
  }
}

impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/create_fake_namespace_object.js").to_string())
  }
}
