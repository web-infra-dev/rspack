use super::mf_scope_runtime_module::MFScopeRuntimeModule;
use crate::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext,
};

#[derive(Debug, Default)]
pub struct MFScopeRuntimePlugin;

impl Plugin for MFScopeRuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.MFScopeRuntimePlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    args
      .compilation
      .add_runtime_module(args.chunk, Box::<MFScopeRuntimeModule>::default());
    Ok(())
  }
}
