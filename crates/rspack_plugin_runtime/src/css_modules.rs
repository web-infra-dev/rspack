use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkLoading, ChunkLoadingType, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::{is_enabled_for_chunk, CssLoadingRuntimeModule};

#[derive(Debug)]
pub struct CssModulesPlugin;

#[async_trait]
impl Plugin for CssModulesPlugin {
  fn name(&self) -> &'static str {
    "CssModulesPlugin"
  }

  fn apply(&self, _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Jsonp);
    let is_enabled_for_chunk = is_enabled_for_chunk(chunk, &chunk_loading_value, compilation);
    let runtime_requirements = &mut args.runtime_requirements;

    if (runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      || runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS))
      && is_enabled_for_chunk
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
