use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkLoading, ChunkLoadingType, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::{is_enabled_for_chunk, ImportScriptsChunkLoadingRuntimeModule};

#[derive(Debug)]
pub struct ImportScriptsChunkLoadingPlugin;

#[async_trait]
impl Plugin for ImportScriptsChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "ImportScriptsChunkLoadingPlugin"
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
    let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::ImportScripts);
    let is_enabled_for_chunk = is_enabled_for_chunk(chunk, &chunk_loading_value, compilation);
    let runtime_requirements = &mut args.runtime_requirements;

    let mut has_chunk_loading = false;
    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
          has_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
        }
        RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS if is_enabled_for_chunk => {
          has_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
          runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
          runtime_requirements.insert(RuntimeGlobals::HMR_MODULE_DATA);
          runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
        }
        RuntimeGlobals::HMR_DOWNLOAD_MANIFEST if is_enabled_for_chunk => {
          has_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::PUBLIC_PATH);
          runtime_requirements.insert(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME);
        }
        RuntimeGlobals::BASE_URI if is_enabled_for_chunk => {
          has_chunk_loading = true;
        }
        _ => {}
      }

      if has_chunk_loading && is_enabled_for_chunk {
        runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
        runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
        let with_create_script_url = compilation.options.output.trusted_types.is_some();
        if with_create_script_url {
          runtime_requirements.insert(RuntimeGlobals::CREATE_SCRIPT_URL);
        }
        compilation.add_runtime_module(
          chunk,
          ImportScriptsChunkLoadingRuntimeModule::new(
            **runtime_requirements,
            with_create_script_url,
          )
          .boxed(),
        );
      }
    }
    Ok(())
  }
}
