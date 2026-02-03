use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

const DEFINE_PROPERTY_GETTERS_TEMPLATE: &str = include_str!("runtime/define_property_getters.ejs");
const DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(DEFINE_PROPERTY_GETTERS_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct DefinePropertyGettersRuntimeModule {
  id: Identifier,
}

impl DefinePropertyGettersRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}define_property_getters",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for DefinePropertyGettersRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      DEFINE_PROPERTY_GETTERS_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *DEFINE_PROPERTY_GETTERS_RUNTIME_REQUIREMENTS
  }
}
