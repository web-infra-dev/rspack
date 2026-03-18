use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use rspack_core::{
  RuntimeGlobals, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::{Result, error};
use rustc_hash::{FxHashMap, FxHashSet};

const SHARED_LAYER_SEPARATOR: &str = "\u{0000}";

fn split_shared_lookup_key(shared_key: &str) -> (&str, Option<&str>) {
  match shared_key.split_once(SHARED_LAYER_SEPARATOR) {
    Some((share_key, layer)) => (share_key, Some(layer)),
    None => (shared_key, None),
  }
}

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

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    if self.shared_used_exports.is_empty() {
      return Ok(String::new());
    }
    let federation_global = format!(
      "{}.federation",
      context
        .runtime_template
        .render_runtime_globals(&RuntimeGlobals::REQUIRE)
    );
    let mut merged_exports = FxHashMap::<String, FxHashSet<String>>::default();
    for (shared_lookup_key, set) in self.shared_used_exports.iter() {
      let (share_key, _) = split_shared_lookup_key(shared_lookup_key);
      let merged_set = merged_exports.entry(share_key.to_string()).or_default();
      merged_set.extend(set.iter().cloned());
    }
    // Convert set to vec for JSON serialization stability
    let stable_map: BTreeMap<String, Vec<String>> = merged_exports
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
