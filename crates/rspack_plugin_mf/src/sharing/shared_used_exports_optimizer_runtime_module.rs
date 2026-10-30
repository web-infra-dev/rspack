use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use rspack_core::{
  Compilation, RuntimeModule, RuntimeModuleGenerateContext, RuntimeModuleStage, RuntimeTemplate,
  impl_runtime_module,
};
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  SharedIdentity,
  utils::{runtime_require_scope_name, runtime_require_scope_requirement},
};

#[impl_runtime_module]
#[derive(Debug)]
pub struct SharedUsedExportsOptimizerRuntimeModule {
  used_exports_json: Option<String>,
}

impl SharedUsedExportsOptimizerRuntimeModule {
  pub(crate) fn new(
    runtime_template: &RuntimeTemplate,
    shared_used_exports: Arc<FxHashMap<SharedIdentity, FxHashSet<String>>>,
  ) -> Self {
    let mut merged_exports = FxHashMap::<String, FxHashSet<String>>::default();
    for (identity, set) in shared_used_exports.iter() {
      merged_exports
        .entry(identity.share_key.clone())
        .or_default()
        .extend(set.iter().cloned());
    }
    let used_exports_json = if merged_exports.is_empty() {
      None
    } else {
      let stable_map: BTreeMap<String, Vec<String>> = merged_exports
        .iter()
        .map(|(share_key, set)| {
          let mut exports: Vec<String> = set.iter().cloned().collect();
          exports.sort_unstable();
          (share_key.clone(), exports)
        })
        .collect();
      Some(
        simd_json::to_string(&stable_map)
          .expect("shared used exports contain only serializable strings"),
      )
    };
    Self::with_name(
      runtime_template,
      "module_federation/shared_used_exports",
      used_exports_json,
    )
  }
}

#[async_trait]
impl RuntimeModule for SharedUsedExportsOptimizerRuntimeModule {
  fn runtime_module_variables() -> &'static [&'static str] {
    &[]
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn should_isolate(&self, _runtime_mode: rspack_core::runtime_mode::RuntimeMode) -> bool {
    true
  }

  fn runtime_requirements(
    &self,
    compilation: &Compilation,
  ) -> rspack_core::RuntimeModuleRuntimeRequirements {
    rspack_core::RuntimeModuleRuntimeRequirements {
      dependencies: { runtime_require_scope_requirement(compilation) },
      ..Default::default()
    }
  }

  async fn generate(&self, context: &RuntimeModuleGenerateContext<'_>) -> Result<String> {
    let Some(used_exports_json) = &self.used_exports_json else {
      return Ok(String::new());
    };
    let federation_global = format!(
      "{}.federation",
      runtime_require_scope_name(context.runtime_template)
    );
    Ok(format!(
      r#"
if(!{federation_global}){{return;}}
{federation_global}.usedExports = {used_exports_json};
"#
    ))
  }
}
