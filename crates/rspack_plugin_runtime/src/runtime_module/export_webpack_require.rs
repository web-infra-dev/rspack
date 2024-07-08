use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  Compilation, RuntimeModule,
};
use rspack_identifier::Identifier;

#[impl_runtime_module]
#[derive(Debug, Default)]
pub struct ExportWebpackRequireRuntimeModule {
  id: Identifier,
}

impl ExportWebpackRequireRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/export_webpack_runtime"))
  }
}

impl RuntimeModule for ExportWebpackRequireRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    Ok(RawSource::from("export default __webpack_require__;").boxed())
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
