use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

#[impl_runtime_module]
#[derive(Debug)]
pub struct CompatGetDefaultExportRuntimeModule {
  id: Identifier,
}

impl CompatGetDefaultExportRuntimeModule {
  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(include_str!("runtime/compat_get_default_export.js").to_string())
  }
}

impl Default for CompatGetDefaultExportRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from(
      "webpack/runtime/compat_get_default_export",
    ))
  }
}

impl RuntimeModule for CompatGetDefaultExportRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }
}
