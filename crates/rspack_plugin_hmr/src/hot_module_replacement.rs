use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};
use rspack_plugin_runtime::extract_runtime_globals_from_ejs;
use rspack_util::test::is_hot_test;

static HOT_MODULE_REPLACEMENT_TEMPLATE: &str = include_str!("runtime/hot_module_replacement.ejs");
static HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(HOT_MODULE_REPLACEMENT_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct HotModuleReplacementRuntimeModule {
  id: Identifier,
}

impl HotModuleReplacementRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}hot_module_replacement",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for HotModuleReplacementRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      HOT_MODULE_REPLACEMENT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let content = compilation.runtime_template.render(
      self.id.as_str(),
      Some(serde_json::json!({
        "_is_hot_test": is_hot_test(),
      })),
    )?;

    Ok(content)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *HOT_MODULE_REPLACEMENT_RUNTIME_REQUIREMENTS
  }
}
