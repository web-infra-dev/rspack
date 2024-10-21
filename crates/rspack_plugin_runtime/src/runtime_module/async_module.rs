use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncRuntimeModule {
  id: Identifier,
}

impl AsyncRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/async_module.js").to_string())
  }
}

impl Default for AsyncRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/async_module"))
  }
}

impl RuntimeModule for AsyncRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
}
