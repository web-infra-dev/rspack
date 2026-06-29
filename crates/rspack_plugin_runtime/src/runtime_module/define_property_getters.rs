use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

static DEFINE_PROPERTY_GETTERS_TEMPLATE: &str = include_str!("runtime/define_property_getters.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefinePropertyGettersRuntimeModule {}

impl DefinePropertyGettersRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for DefinePropertyGettersRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::HAS_OWN_PROPERTY,
      write: { RuntimeGlobals::DEFINE_PROPERTY_GETTERS },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      DEFINE_PROPERTY_GETTERS_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let define_property_code = if context
      .compilation
      .options
      .output
      .environment
      .supports_computed_property()
    {
      "Object.defineProperty(exports, key, { enumerable: true, [kind]: defs[key] });"
    } else {
      "var descriptor = { enumerable: true };\n\t\t\t\tdescriptor[kind] = defs[key];\n\t\t\t\tObject.defineProperty(exports, key, descriptor);"
    };
    let source = context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_define_property_code": define_property_code,
      })),
    )?;

    Ok(source)
  }
}
