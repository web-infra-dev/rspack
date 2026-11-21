use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct MakeNamespaceObjectRuntimeModule {}

impl MakeNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for MakeNamespaceObjectRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      define: { RuntimeGlobals::MAKE_NAMESPACE_OBJECT },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      include_str!("runtime/make_namespace_object.ejs").to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let environment = &context.compilation.options.output.environment;
    let supports_symbol = environment.supports_symbol();
    let source = context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_define_properties": supports_symbol && environment.supports_computed_property(),
        "_supports_symbol": supports_symbol,
      })),
    )?;

    Ok(source)
  }
}
