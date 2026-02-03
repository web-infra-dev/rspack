use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

const CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE: &str =
  include_str!("runtime/create_fake_namespace_object.ejs");
const CREATE_FAKE_NAMESPACE_OBJECT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct CreateFakeNamespaceObjectRuntimeModule {
  id: Identifier,
}

impl CreateFakeNamespaceObjectRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}create_fake_namespace_object",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CreateFakeNamespaceObjectRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      CREATE_FAKE_NAMESPACE_OBJECT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *CREATE_FAKE_NAMESPACE_OBJECT_RUNTIME_REQUIREMENTS
  }
}
