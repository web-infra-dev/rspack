use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module, runtime_mode::RuntimeMode,
};

pub static EXPORT_REQUIRE_RUNTIME_MODULE_ID: &str = "export_webpack_require";
pub static EXPORT_REQUIRE_RSPACK_RUNTIME_MODULE_ID: &str = "export_require";

#[impl_runtime_module]
#[derive(Debug)]
pub struct ExportRequireRuntimeModule {}

impl ExportRequireRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    let name = if runtime_template.render_mode().is_legacy() {
      EXPORT_REQUIRE_RUNTIME_MODULE_ID
    } else {
      EXPORT_REQUIRE_RSPACK_RUNTIME_MODULE_ID
    };
    Self::with_name(runtime_template, name)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ExportRequireRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let export_name = context
      .runtime_template
      .render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE);
    let export_temp_name = format!("{export_name}temp");
    Ok(format!(
      r#"var {export_temp_name} = {export_name};
export {{ {export_temp_name} as {export_name} }};
"#,
    ))
  }

  fn should_isolate(&self, _runtime_mode: RuntimeMode) -> bool {
    false
  }
}
