use rspack_core::{BoxModule, ModuleFactoryCreateData, NormalModuleFactoryAfterFactorize, Plugin};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::node_builtins::is_node_builtin;

#[plugin]
#[derive(Debug)]
pub struct EsmNodeTargetPlugin;

impl EsmNodeTargetPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

/// Fix up external type for node-builtin externals.
/// When user configures e.g. `{ 'node:fs': 'node:path' }`, the user's ExternalsPlugin
/// creates the module with correct request but may use an incorrect type (e.g. "var").
/// This hook sets the correct type based on the dependency category:
/// - ESM dependency → "module-import"
/// - non-ESM dependency → "node-commonjs"
/// stage = -10 ensures this runs BEFORE EsmLibraryPlugin's after_factorize (stage 0)
/// which calls set_id based on the external type.
#[plugin_hook(NormalModuleFactoryAfterFactorize for EsmNodeTargetPlugin, stage = -10)]
async fn after_factorize(
  &self,
  data: &mut ModuleFactoryCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if let Some(external_module) = module.as_external_module_mut() {
    let request = external_module.get_request().primary().to_string();
    if is_node_builtin(&request) {
      let dep = data.dependencies[0]
        .as_module_dependency()
        .expect("should be module dependency");
      let expected_type = if dep.category().to_string() == "esm" {
        "module-import"
      } else {
        "node-commonjs"
      };
      if external_module.get_external_type() != expected_type {
        external_module.set_external_type(expected_type.to_string());
      }
    }
  }
  Ok(())
}

impl Plugin for EsmNodeTargetPlugin {
  fn name(&self) -> &'static str {
    "rspack.EsmNodeTargetPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .after_factorize
      .tap(after_factorize::new(self));
    Ok(())
  }
}
