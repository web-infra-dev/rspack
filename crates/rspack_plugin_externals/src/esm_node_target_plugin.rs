use rspack_core::{
  BoxModule, DependencyType, ModuleFactoryCreateData, NormalModuleFactoryAfterFactorize, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::node_builtins::is_node_builtin;

#[plugin]
#[derive(Debug)]
pub struct EsmNodeTargetPlugin;

impl Default for EsmNodeTargetPlugin {
  fn default() -> Self {
    Self::new()
  }
}

impl EsmNodeTargetPlugin {
  pub fn new() -> Self {
    Self::new_inner()
  }
}

/// Fix up external type for node-builtin externals when a CJS require
/// imports a "module" or "module-import" type external.
/// For example, `externals: { 'node:fs': 'module node:path' }` used from
/// `require('node:fs')` should be downgraded to "node-commonjs".
/// stage = -10 ensures this runs BEFORE EsmLibraryPlugin's after_factorize (stage 0)
/// which calls set_id based on the external type.
#[plugin_hook(NormalModuleFactoryAfterFactorize for EsmNodeTargetPlugin, stage = -10)]
async fn after_factorize(
  &self,
  data: &mut ModuleFactoryCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  if let Some(external_module) = module.as_external_module_mut()
    && external_module.get_external_type().starts_with("module")
  {
    let request = external_module.get_request().primary().to_string();
    if is_node_builtin(&request) {
      let dep = data.dependencies[0]
        .as_module_dependency()
        .expect("should be module dependency");
      if matches!(
        dep.dependency_type(),
        DependencyType::CjsRequire | DependencyType::CjsFullRequire
      ) {
        external_module.set_external_type("node-commonjs".to_string());
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
