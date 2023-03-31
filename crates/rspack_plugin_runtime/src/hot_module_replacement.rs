use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::HotModuleReplacementRuntimeModule;

#[derive(Debug)]
pub struct HotModuleReplacementPlugin {}

#[async_trait]
impl Plugin for HotModuleReplacementPlugin {
  fn name(&self) -> &'static str {
    "HotModuleReplacementPlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    // TODO: the hmr runtime is depend on module.id, but webpack not add it.
    runtime_requirements.insert(RuntimeGlobals::MODULE_ID);
    runtime_requirements.insert(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST);
    runtime_requirements.insert(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
    runtime_requirements.insert(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
    compilation.add_runtime_module(chunk, HotModuleReplacementRuntimeModule::default().boxed());

    Ok(())
  }
}
