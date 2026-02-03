use std::sync::LazyLock;

use rspack_collections::Identifier;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeTemplate, impl_runtime_module,
};

use crate::extract_runtime_globals_from_ejs;

static COMPAT_GET_DEFAULT_EXPORT_TEMPLATE: &str =
  include_str!("runtime/compat_get_default_export.ejs");
static COMPAT_GET_DEFAULT_EXPORT_RUNTIME_REQUIREMENTS: LazyLock<RuntimeGlobals> =
  LazyLock::new(|| extract_runtime_globals_from_ejs(COMPAT_GET_DEFAULT_EXPORT_TEMPLATE));

#[impl_runtime_module]
#[derive(Debug)]
pub struct CompatGetDefaultExportRuntimeModule {
  id: Identifier,
}

impl CompatGetDefaultExportRuntimeModule {
  pub fn new(runtime_template: &RuntimeTemplate) -> Self {
    Self::with_default(Identifier::from(format!(
      "{}compat_get_default_export",
      runtime_template.runtime_module_prefix()
    )))
  }
}

#[async_trait::async_trait]
impl RuntimeModule for CompatGetDefaultExportRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn template(&self) -> Vec<(String, String)> {
    vec![(
      self.id.to_string(),
      COMPAT_GET_DEFAULT_EXPORT_TEMPLATE.to_string(),
    )]
  }

  async fn generate(&self, compilation: &Compilation) -> rspack_error::Result<String> {
    let source = compilation.runtime_template.render(&self.id, None)?;

    Ok(source)
  }

  fn additional_runtime_requirements(&self, _compilation: &Compilation) -> RuntimeGlobals {
    *COMPAT_GET_DEFAULT_EXPORT_RUNTIME_REQUIREMENTS
  }
}
