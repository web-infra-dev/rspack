use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext,
  RuntimeModuleRuntimeRequirements, RuntimeTemplate, impl_runtime_module,
  runtime_mode::RuntimeMode,
};
use rspack_plugin_runtime::extract_runtime_globals_dependencies_from_ejs;
use rspack_util::test::is_hot_test;

static HOT_MODULE_REPLACEMENT_TEMPLATE: &str = include_str!("runtime/hot_module_replacement.ejs");
static HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeModuleRuntimeRequirements> =
  LazyLock::new(|| RuntimeModuleRuntimeRequirements {
    dependencies: extract_runtime_globals_dependencies_from_ejs(
      HOT_MODULE_REPLACEMENT_TEMPLATE,
      RuntimeGlobals::default(),
    ),
    write: RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
      | RuntimeGlobals::HMR_MODULE_DATA
      | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
      | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS,
    ..Default::default()
  });

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementRuntimeModule {}

impl HotModuleReplacementRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      HOT_MODULE_REPLACEMENT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let content = context.runtime_template.render(
      self.id().as_str(),
      Some(serde_json::json!({
        "_is_hot_test": is_hot_test(),
        "_is_rspack_runtime_mode": context.compilation.options.experiments.runtime_mode == RuntimeMode::Rspack,
      })),
    )?;

    Ok(content)
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS.dependencies,
      write: {
        RuntimeGlobals::INTERCEPT_MODULE_EXECUTION
          | RuntimeGlobals::HMR_MODULE_DATA
          | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
          | RuntimeGlobals::HMR_INVALIDATE_MODULE_HANDLERS
      },
      ..Default::default()
    }
  }
}
