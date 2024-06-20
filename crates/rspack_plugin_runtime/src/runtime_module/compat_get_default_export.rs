use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct CompatGetDefaultExportRuntimeModule {
  id: Identifier,
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

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from(include_str!("runtime/compat_get_default_export.js")).boxed())
  }
}
