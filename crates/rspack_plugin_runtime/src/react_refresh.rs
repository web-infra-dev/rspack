use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::ReactRefreshRuntimeModule;

#[derive(Debug)]
pub struct ReactRefreshPlugin;

#[async_trait]
impl Plugin for ReactRefreshPlugin {
  fn name(&self) -> &'static str {
    "ReactRefreshPlugin"
  }

  fn apply(&self, _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
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

    runtime_requirements.insert(RuntimeGlobals::INTERCEPT_MODULE_EXECUTION);
    runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
    compilation.add_runtime_module(chunk, ReactRefreshRuntimeModule::default().boxed());

    Ok(())
  }
}
