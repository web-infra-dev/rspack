use async_trait::async_trait;
use rspack_core::{
  ChunkLoading, ChunkLoadingType, Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput,
  RuntimeGlobals, RuntimeRequirementsInTreeArgs,
};

use crate::runtime_module::{is_enabled_for_chunk, CssLoadingRuntimeModule};

#[derive(Debug)]
pub struct CssModulesPlugin;

#[async_trait]
impl Plugin for CssModulesPlugin {
  fn name(&self) -> &'static str {
    "CssModulesPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Jsonp);
    let is_enabled_for_chunk = is_enabled_for_chunk(chunk, &chunk_loading_value, compilation);
    let runtime_requirements = args.runtime_requirements;
    let runtime_requirements_mut = &mut args.runtime_requirements_mut;

    if (runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      || runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS))
      && is_enabled_for_chunk
    {
      runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
      runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_CSS_FILENAME);
      runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      compilation.add_runtime_module(chunk, Box::<CssLoadingRuntimeModule>::default());
    }

    Ok(())
  }
}
