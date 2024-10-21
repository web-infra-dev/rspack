use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct HasOwnPropertyRuntimeModule {
  id: Identifier,
}

impl HasOwnPropertyRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/has_own_property.js").to_string())
  }
}

impl Default for HasOwnPropertyRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/has_own_property"))
  }
}

impl RuntimeModule for HasOwnPropertyRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
}
