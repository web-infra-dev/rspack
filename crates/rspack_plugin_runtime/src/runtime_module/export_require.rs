use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module, runtime_mode::RuntimeMode,
};

pub static EXPORT_REQUIRE_RUNTIME_MODULE_ID: &str = "export_webpack_require";
pub static EXPORT_REQUIRE_RSPACK_RUNTIME_MODULE_ID: &str = "export_require";
const RUNTIME_MODULE_VARIABLES: &[&str] = &[
  "__webpack_require__temp",
  "__rspack_contexttemp",
  "rspackRequiretemp",
];

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
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::REQUIRE_SCOPE,
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/export_require.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    context.runtime_template.render(self.id(), None)
  }

  fn should_isolate(&self, _runtime_mode: RuntimeMode) -> bool {
    false
  }
}
