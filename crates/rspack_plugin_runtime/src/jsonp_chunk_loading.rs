use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::JsonpChunkLoadingRuntimeModule;

#[derive(Debug)]
pub struct JsonpChunkLoadingPlugin {}

#[async_trait]
impl Plugin for JsonpChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "JsonpChunkLoadingPlugin"
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

    let mut has_jsonp_chunk_loading = false;
    for runtime_requirement in runtime_requirements.clone().iter() {
      match runtime_requirement.as_str() {
        runtime_globals::ENSURE_CHUNK_HANDLERS => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(runtime_globals::PUBLIC_PATH.to_string());
          runtime_requirements.insert(runtime_globals::LOAD_SCRIPT.to_string());
          runtime_requirements.insert(runtime_globals::GET_CHUNK_SCRIPT_FILENAME.to_string());
        }
        runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(runtime_globals::PUBLIC_PATH.to_string());
          runtime_requirements.insert(runtime_globals::LOAD_SCRIPT.to_string());
          runtime_requirements
            .insert(runtime_globals::GET_CHUNK_UPDATE_SCRIPT_FILENAME.to_string());
          runtime_requirements.insert(runtime_globals::MODULE_CACHE.to_string());
          runtime_requirements.insert(runtime_globals::HMR_MODULE_DATA.to_string());
          runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY.to_string());
        }
        runtime_globals::HMR_DOWNLOAD_MANIFEST => {
          has_jsonp_chunk_loading = true;
          runtime_requirements.insert(runtime_globals::PUBLIC_PATH.to_string());
          runtime_requirements.insert(runtime_globals::GET_UPDATE_MANIFEST_FILENAME.to_string());
        }
        runtime_globals::ON_CHUNKS_LOADED => {
          has_jsonp_chunk_loading = true;
        }
        _ => {}
      }

      if has_jsonp_chunk_loading {
        runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY.to_string());
        runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY.to_string());
        compilation.add_runtime_module(
          chunk,
          JsonpChunkLoadingRuntimeModule::new(runtime_requirements.clone()).boxed(),
        );
      }
    }
    Ok(())
  }
}
