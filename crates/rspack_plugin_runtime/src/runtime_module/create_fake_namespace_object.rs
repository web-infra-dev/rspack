use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module, runtime_mode::RuntimeMode,
};

use crate::extract_runtime_module_variables_from_ejs;

static CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/create_fake_namespace_object.ejs");
// No `__proto__` fallback: `Object.create` and `Object.defineProperty` in the
// generated helper already require the same ES5 baseline.
static RUNTIME_MODULE_VARIABLES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
  extract_runtime_module_variables_from_ejs(&[CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE])
});

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
  fn runtime_module_variables() -> &'static [&'static str] {
    RUNTIME_MODULE_VARIABLES.as_slice()
  }

  fn runtime_requirements(
    &self,
    _compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: RuntimeGlobals::MAKE_NAMESPACE_OBJECT | RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      define: { RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT },
      ..Default::default()
    }
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id().to_string(),
      CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(
    &self,
    context: &RuntimeModuleGenerateContext<'_>,
  ) -> rspack_error::Result<String> {
    let this = if context.compilation.options.experiments.runtime_mode == RuntimeMode::Rspack {
      "(typeof this === \"function\" ? this : this.r)"
    } else {
      "this"
    };
    let params = Some(serde_json::json!({
      "__this": this,
      "_leaf_prototypes_assignment": context.runtime_template.assign_or(
        "leafPrototypes",
        "[null, getProto({}), getProto([]), getProto(getProto)]",
      ),
    }));
    let mut source = context.runtime_template.render(self.id(), params)?;

    let trimmed_len = source.trim_end().len();
    source.truncate(trimmed_len);
    Ok(source)
  }
}
