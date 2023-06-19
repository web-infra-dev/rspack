use async_trait::async_trait;
use rspack_core::{
  is_enabled_for_chunk, AdditionalChunkRuntimeRequirementsArgs, ChunkLoading, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::JsonpChunkLoadingRuntimeModule;

#[derive(Debug)]
pub struct JsonpChunkLoadingPlugin;

#[async_trait]
impl Plugin for JsonpChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "JsonpChunkLoadingPlugin"
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
    let chunk_loading_value = ChunkLoading::Jsonp;
    let is_enabled_for_chunk = is_enabled_for_chunk(chunk, &chunk_loading_value, compilation);
    let runtime_requirements = &mut args.runtime_requirements;

    let mut has_jsonp_chunk_loading = false;
    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::LOAD_SCRIPT);
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
        }
        RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS if is_enabled_for_chunk => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::LOAD_SCRIPT);
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
          runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
          runtime_requirements.insert(RuntimeGlobals::HMR_MODULE_DATA);
          runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
        }
        RuntimeGlobals::HMR_DOWNLOAD_MANIFEST if is_enabled_for_chunk => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME);
        }
        RuntimeGlobals::ON_CHUNKS_LOADED | RuntimeGlobals::BASE_URI if is_enabled_for_chunk => {
          has_jsonp_chunk_loading = true;
        }
        _ => {}
      }

      if has_jsonp_chunk_loading && is_enabled_for_chunk {
        runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
        runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
        compilation.add_runtime_module(
          chunk,
          JsonpChunkLoadingRuntimeModule::new(**runtime_requirements).boxed(),
        );
      }
    }
    Ok(())
  }
}
