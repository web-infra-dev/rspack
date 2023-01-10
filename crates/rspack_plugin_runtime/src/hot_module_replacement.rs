use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
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
    runtime_requirements.insert(runtime_globals::MODULE_ID.to_string());
    runtime_requirements.insert(runtime_globals::HMR_DOWNLOAD_MANIFEST.to_string());
    runtime_requirements.insert(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS.to_string());
    runtime_requirements.insert(runtime_globals::INTERCEPT_MODULE_EXECUTION.to_string());
    runtime_requirements.insert(runtime_globals::MODULE_CACHE.to_string());
    compilation.add_runtime_module(chunk, HotModuleReplacementRuntimeModule::default().boxed());

    Ok(())
  }
}
