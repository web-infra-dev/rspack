use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeGlobalsRenderMode, RuntimeModule,
  RuntimeModuleGenerateContext, RuntimeTemplate, RuntimeVariable, impl_runtime_module,
};

use crate::extract_runtime_module_variables_from_ejs;

static ASYNC_MODULE_TEMPLATE: &str = include_str!("runtime/async_module.ejs");
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> =
  LazyLock::new(|| extract_runtime_module_variables_from_ejs(&[ASYNC_MODULE_TEMPLATE]));

#[impl_runtime_module(runtime_module_variables)]
#[derive(Debug)]
pub struct AsyncRuntimeModule {}

impl AsyncRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, "async_module")
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let runtime_template = context.runtime_template;
    let uses_lexical_runtime_globals = match runtime_template.render_mode() {
      RuntimeGlobalsRenderMode::RspackLexical | RuntimeGlobalsRenderMode::RspackExport => true,
      RuntimeGlobalsRenderMode::Webpack | RuntimeGlobalsRenderMode::RspackContext => false,
    };
    runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_module_cache": runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
        "_uses_lexical_runtime_globals": uses_lexical_runtime_globals,
      })),
    )
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), ASYNC_MODULE_TEMPLATE.to_string())]
  }
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { RuntimeGlobals::REQUIRE | RuntimeGlobals::MODULE_CACHE },
      define: {
        RuntimeGlobals::ASYNC_MODULE
          | RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
          | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES
          | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL
      },
      ..Default::default()
    }
  }
}
