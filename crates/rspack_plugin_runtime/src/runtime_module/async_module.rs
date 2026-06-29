use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  RuntimeVariable, impl_runtime_module,
};

static ASYNC_MODULE_TEMPLATE: &str = include_str!("runtime/async_module.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct AsyncRuntimeModule {}

impl AsyncRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_name(runtime_template, "async_module")
  }
}

#[async_trait::async_trait]
impl RuntimeModule for AsyncRuntimeModule {
  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let runtime_template = context.runtime_template;
    runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_module_cache": runtime_template.render_runtime_variable(&RuntimeVariable::ModuleCache),
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
      write: {
        RuntimeGlobals::ASYNC_MODULE
          | RuntimeGlobals::ASYNC_MODULE_EXPORT_SYMBOL
          | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES
          | RuntimeGlobals::DEFERRED_MODULES_ASYNC_TRANSITIVE_DEPENDENCIES_SYMBOL
      },
      ..Default::default()
    }
  }
}
