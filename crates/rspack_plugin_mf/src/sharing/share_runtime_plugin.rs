use async_trait::async_trait;
use rspack_core::{
  Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput, RuntimeGlobals, RuntimeModuleExt,
  RuntimeRequirementsInTreeArgs,
};

use crate::ShareRuntimeModule;

#[derive(Debug)]
pub struct ShareRuntimePlugin {
  enhanced: bool,
}

impl ShareRuntimePlugin {
  pub fn new(enhanced: bool) -> Self {
    Self { enhanced }
  }
}

#[async_trait]
impl Plugin for ShareRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareRuntimePlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::SHARE_SCOPE_MAP)
    {
      args
        .compilation
        .add_runtime_module(args.chunk, ShareRuntimeModule::new(self.enhanced).boxed());
    }
    Ok(())
  }
}
