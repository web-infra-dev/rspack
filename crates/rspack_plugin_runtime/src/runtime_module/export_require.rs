use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

const EXPORT_TEMP_NAME: &str = "__webpack_require_temp__";

pub static EXPORT_REQUIRE_RUNTIME_MODULE_ID: LazyLock<Identifier> =
  LazyLock::new(|| Identifier::from("webpack/runtime/export_webpack_require"));

#[impl_runtime_module]
#[derive(Debug, Default)]
pub struct ExportRequireRuntimeModule {
  id: Identifier,
}

impl ExportRequireRuntimeModule {
  pub fn new() -> Self {
    Self::with_default(*EXPORT_REQUIRE_RUNTIME_MODULE_ID)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ExportRequireRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    Ok(format!(
      "var {EXPORT_TEMP_NAME} = {};\nexport {{ {EXPORT_TEMP_NAME} as {} }};\n",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE),
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    ))
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
