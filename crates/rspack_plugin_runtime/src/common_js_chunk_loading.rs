use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::ReadFileChunkLoadingRuntimeModule;
use crate::runtime_module::RequireChunkLoadingRuntimeModule;

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

    let mut has_chunk_loading = false;
    for &runtime_requirement in runtime_requirements.clone().iter() {
      match runtime_requirement {
        runtime_globals::ENSURE_CHUNK_HANDLERS => {
          has_chunk_loading = true;
          runtime_requirements.insert(runtime_globals::GET_CHUNK_SCRIPT_FILENAME);
        }
        runtime_globals::HMR_DOWNLOAD_UPDATE_HANDLERS => {
          runtime_requirements.insert(runtime_globals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
          runtime_requirements.insert(runtime_globals::MODULE_CACHE);
          runtime_requirements.insert(runtime_globals::HMR_MODULE_DATA);
          runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY);
          has_chunk_loading = true;
        }
        runtime_globals::HMR_DOWNLOAD_MANIFEST => {
          has_chunk_loading = true;
          runtime_requirements.insert(runtime_globals::GET_UPDATE_MANIFEST_FILENAME);
        }
        runtime_globals::ON_CHUNKS_LOADED => {
          has_chunk_loading = true;
        }
        runtime_globals::EXTERNAL_INSTALL_CHUNK => {
          has_chunk_loading = true;
        }
        _ => {}
      }
    }

    if has_chunk_loading {
      runtime_requirements.insert(runtime_globals::MODULE_FACTORIES_ADD_ONLY);
      runtime_requirements.insert(runtime_globals::HAS_OWN_PROPERTY);
      compilation.add_runtime_module(
        chunk,
        RequireChunkLoadingRuntimeModule::new(runtime_requirements.clone()).boxed(),
      );
    }

    Ok(())
  }
}
