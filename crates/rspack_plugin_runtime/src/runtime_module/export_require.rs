use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{Compilation, RuntimeGlobals, RuntimeModule, impl_runtime_module};

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
    let require_name = compilation
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let export_temp_name = format!("{require_name}temp");
    Ok(format!(
      "var {export_temp_name} = {require_name};\nexport {{ {export_temp_name} as {require_name} }};\n",
    ))
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
