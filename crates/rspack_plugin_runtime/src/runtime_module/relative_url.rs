use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct RelativeUrlRuntimeModule {
  id: Identifier,
}

impl RelativeUrlRuntimeModule {
  fn generate(&self, _: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/relative_url.js").to_string())
  }
}

impl Default for RelativeUrlRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/relative_url"))
  }
}

impl RuntimeModule for RelativeUrlRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
}
