use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

static REEXPORT_TEMPLATE: &str = include_str!("runtime/reexport.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct ReexportRuntimeModule {}

impl ReexportRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for ReexportRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      define: { RuntimeGlobals::REEXPORT },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(self.id().to_string(), REEXPORT_TEMPLATE.to_string())]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let environment = context.compilation.options.output.environment;
    let supports_const = environment.supports_const();
    let getter = if environment.supports_arrow_function() && supports_const {
      "() => source[key]"
    } else {
      "function(key) { return source[key]; }.bind(0, key)"
    };
    context.runtime_template.render(
      self.id(),
      Some(serde_json::json!({
        "_key_decl": if supports_const { "const" } else { "var" },
        "_getter": getter,
      })),
    )
  }
}
