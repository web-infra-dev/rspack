use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

const EXPORT_TEMP_NAME: &str = "__webpack_require_temp__";

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
    Ok(format!(
      "var {EXPORT_TEMP_NAME} = {};\nexport {{ {EXPORT_TEMP_NAME} as {} }};\n",
      RuntimeGlobals::REQUIRE,
      RuntimeGlobals::REQUIRE
    ))
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
