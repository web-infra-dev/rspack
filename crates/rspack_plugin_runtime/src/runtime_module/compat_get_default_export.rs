use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct CompatGetDefaultExportRuntimeModule {
  id: Identifier,
}

impl Default for CompatGetDefaultExportRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/compat_get_default_export"),
      source_map_kind: SourceMapKind::None,
      custom_source: None,
    }
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
