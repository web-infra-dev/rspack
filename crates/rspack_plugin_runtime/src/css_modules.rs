use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::CssLoadingRuntimeModule;

#[derive(Debug)]
pub struct CssModulesPlugin {}

#[async_trait]
impl Plugin for CssModulesPlugin {
  fn name(&self) -> &'static str {
    "CssModulesPlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(runtime_globals::ENSURE_CHUNK_HANDLERS)
      || runtime_requirements.contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS)
    {
      runtime_requirements.insert(runtime_globals::PUBLIC_PATH);
      runtime_requirements.insert(runtime_globals::GET_CHUNK_CSS_FILENAME);
      runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY);
      runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY);
      compilation.add_runtime_module(
        chunk,
        CssLoadingRuntimeModule::new(runtime_requirements.clone()).boxed(),
      );
    }

    Ok(())
  }
}
