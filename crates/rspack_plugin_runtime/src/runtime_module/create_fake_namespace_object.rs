use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module, runtime_mode::RuntimeMode,
};

static CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/create_fake_namespace_object.ejs");

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {}

impl CreateFakeNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(runtime_template)
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::MAKE_NAMESPACE_OBJECT | RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      write: { RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let params = Some(
      if context.compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
        serde_json::json!({
          "__this": "(typeof this === \"function\" ? this : this.r)"
        })
      } else {
        serde_json::json!({
          "__this": "this"
        })
      },
    );
    let source = context.runtime_template.render(&self.id, params)?;

    Ok(source)
  }
}
