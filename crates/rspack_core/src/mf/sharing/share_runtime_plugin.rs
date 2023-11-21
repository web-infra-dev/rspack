use super::share_runtime_module::ShareRuntimeModule;
use crate::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext,
};

#[derive(Debug, Default)]
pub struct ShareRuntimePlugin;

impl Plugin for ShareRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareRuntimePlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    args
      .compilation
      .add_runtime_module(args.chunk, Box::<ShareRuntimeModule>::default());
    Ok(())
  }
}
