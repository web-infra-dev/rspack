use std::sync::LazyLock;

use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeTemplate,
  impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static DEFINE_PROPERTY_GETTERS_TEMPLATE: &str = include_str!("runtime/define_property_getters.ejs");
static DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(DEFINE_PROPERTY_GETTERS_TEMPLATE));

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
  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
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
      &self.id,
      Some(serde_json::json!({
        "_define_property_code": define_property_code,
      })),
    )?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS
  }
}
