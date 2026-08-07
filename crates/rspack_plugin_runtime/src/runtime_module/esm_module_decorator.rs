use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct ESMModuleDecoratorRuntimeModule {
  include_module_id: bool,
}

impl ESMModuleDecoratorRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate, include_module_id: bool) -> Self {
    Self::with_default(runtime_template, include_module_id)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ESMModuleDecoratorRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::ESM_MODULE_DECORATOR },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/esm_module_decorator.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let assignment_error = if self.include_module_id {
      format!(
        "'ES Modules may not assign module.exports or exports.*, Use ESM export syntax, instead: ' + {}",
        context
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::MODULE_ID)
      )
    } else {
      "'ES Modules may not assign module.exports or exports.*, Use ESM export syntax instead.'"
        .to_string()
    };
    let source = context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({ "_assignment_error": assignment_error })),
    )?;

    Ok(source)
  }
}
