use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

pub static EXPORT_REQUIRE_RUNTIME_MODULE_ID: &str = "export_webpack_require";

#[impl_runtime_module]
#[derive(Debug)]
pub struct ExportRequireRuntimeModule {}

impl ExportRequireRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, EXPORT_REQUIRE_RUNTIME_MODULE_ID)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ExportRequireRuntimeModule {
  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let require_name = compilation
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE);
    let export_temp_name = format!("{require_name}temp");
    Ok(format!(
      r#"var {export_temp_name} = {require_name};
export {{ {export_temp_name} as {require_name} }};
"#,
    ))
  }

  fn should_isolate(&self) -> bool {
    false
  }
}
