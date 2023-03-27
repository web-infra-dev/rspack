use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext, RuntimeGlobals, RuntimeModuleExt,
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

    if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      || runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
    {
      runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
      runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
      runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      compilation.add_runtime_module(
        chunk,
        CssLoadingRuntimeModule::new(**runtime_requirements).boxed(),
      );
    }

    Ok(())
  }
}
