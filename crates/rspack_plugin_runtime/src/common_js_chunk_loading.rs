use crate::runtime_module::CommonJsChunkLoadingRuntimeModule;
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};
use rspack_error::Result;

#[derive(Debug)]
pub struct CommonJsChunkLoadingPlugin {}

#[async_trait]
impl Plugin for CommonJsChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "CommonJsChunkLoadingPlugin"
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

    if runtime_requirements.contains(runtime_globals::HMR_DOWNLOAD_MANIFEST) {
      runtime_requirements.insert(runtime_globals::GET_UPDATE_MANIFEST_FILENAME.to_string());
    }

    if runtime_requirements.contains(runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
      runtime_requirements.insert(runtime_globals::GET_CHUNK_UPDATE_SCRIPT_FILENAME.to_string());
      runtime_requirements.insert(runtime_globals::MODULE_CACHE.to_string());
      runtime_requirements.insert(runtime_globals::HMR_MODULE_DATA.to_string());
      runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY.to_string());
    }

    if runtime_requirements.contains(runtime_globals::ENSURE_CHUNK_HANDLERS) {
      runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY.to_string());
      runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY.to_string());
      runtime_requirements.insert(runtime_globals::GET_CHUNK_SCRIPT_FILENAME.to_string());
      compilation.add_runtime_module(chunk, CommonJsChunkLoadingRuntimeModule::default().boxed());
    }

    Ok(())
  }
}
