use rspack_collections::Identifier;
use rspack_core::{impl_runtime_module, Compilation, RuntimeModule};

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

#[async_trait::async_trait]
impl RuntimeModule for ExportWebpackRequireRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, _compilation: &Compilation) -> rspack_error::Result<String> {
    Ok("export default __webpack_require__;".to_string())
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
