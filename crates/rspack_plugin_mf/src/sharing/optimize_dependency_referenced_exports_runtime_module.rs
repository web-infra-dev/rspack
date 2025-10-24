use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use rspack_collections::Identifier;
use rspack_core::{
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, impl_runtime_module,
};
use rspack_error::{Result, error};

#[impl_runtime_module]
#[derive(Debug)]
pub struct OptimizeDependencyReferencedExportsRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
  shared_used_exports: Arc<BTreeMap<String, BTreeMap<String, Vec<String>>>>,
}

impl OptimizeDependencyReferencedExportsRuntimeModule {
  pub fn new(shared_used_exports: Arc<BTreeMap<String, BTreeMap<String, Vec<String>>>>) -> Self {
    Self::with_default(
      Identifier::from("module_federation/shared_used_exports"),
      None,
      shared_used_exports,
    )
  }
}

#[async_trait]
impl RuntimeModule for OptimizeDependencyReferencedExportsRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, _compilation: &Compilation) -> Result<String> {
    if self.shared_used_exports.is_empty() {
      return Ok(String::new());
    }
    let federation_global = format!("{}.federation", RuntimeGlobals::REQUIRE);
    let used_exports_json = serde_json::to_string(&*self.shared_used_exports).map_err(|err| {
      error!(
        "OptimizeDependencyReferencedExportsRuntimeModule: failed to serialize used exports: {err}"
      )
    })?;
    Ok(format!(
      r#"
if(!{federation_global}){{return;}}
{federation_global}.usedExports = {used_exports_json};
"#,
      federation_global = federation_global,
      used_exports_json = used_exports_json
    ))
  }
}
