use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use rspack_core::{
  Compilation, RuntimeGlobals, RuntimeModule, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, error};
use rustc_hash::{FxHashMap, FxHashSet};

#[impl_runtime_module]
#[derive(Debug)]
pub struct SharedUsedExportsOptimizerRuntimeModule {
  // Keep type consistent with plugin: FxHashMap<String, FxHashSet<String>>
  shared_used_exports: Arc<FxHashMap<String, FxHashSet<String>>>,
}

impl SharedUsedExportsOptimizerRuntimeModule {
  pub fn new(
    runtime_template: &RuntimeTemplate,
    shared_used_exports: Arc<FxHashMap<String, FxHashSet<String>>>,
  ) -> Self {
    Self::with_name(
      runtime_template,
      "module_federation/shared_used_exports",
      shared_used_exports,
    )
  }
}

#[async_trait]
impl RuntimeModule for SharedUsedExportsOptimizerRuntimeModule {
  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  async fn generate(&self, compilation: &Compilation) -> Result<String> {
    if self.shared_used_exports.is_empty() {
      return Ok(String::new());
    }
    let federation_global = format!(
      "{}.federation",
      compilation
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    );
    // Convert set to vec for JSON serialization stability
    let stable_map: BTreeMap<String, Vec<String>> = self
      .shared_used_exports
      .iter()
      .map(|(share_key, set)| {
        let mut v: Vec<String> = set.iter().cloned().collect();
        v.sort();
        (share_key.clone(), v)
      })
      .collect();
    let used_exports_json = serde_json::to_string(&stable_map).map_err(|err| {
      error!(
        "OptimizeDependencyReferencedExportsRuntimeModule: failed to serialize used exports: {err}"
      )
    })?;
    Ok(format!(
      r#"
if(!{federation_global}){{return;}}
{federation_global}.usedExports = {used_exports_json};
"#
    ))
  }
}
