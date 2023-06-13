use async_trait::async_trait;
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::ReadFileChunkLoadingRuntimeModule;
use crate::runtime_module::RequireChunkLoadingRuntimeModule;

#[derive(Debug)]
pub struct CommonJsChunkLoadingPlugin {
  pub async_chunk_loading: bool,
}

#[async_trait]
impl Plugin for CommonJsChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "CommonJsChunkLoadingPlugin"
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
    let runtime_requirements = &mut args.runtime_requirements;

    let mut has_chunk_loading = false;
    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS => {
          has_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
        }
        RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS => {
          runtime_requirements.insert(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
          runtime_requirements.insert(RuntimeGlobals::MODULE_CACHE);
          runtime_requirements.insert(RuntimeGlobals::HMR_MODULE_DATA);
          runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
          has_chunk_loading = true;
        }
        RuntimeGlobals::HMR_DOWNLOAD_MANIFEST => {
          has_chunk_loading = true;
          runtime_requirements.insert(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME);
        }
        RuntimeGlobals::ON_CHUNKS_LOADED
        | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
        | RuntimeGlobals::BASE_URI => {
          has_chunk_loading = true;
        }
        _ => {}
      }
    }

    if has_chunk_loading {
      runtime_requirements.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      runtime_requirements.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      if self.async_chunk_loading {
        compilation.add_runtime_module(
          chunk,
          ReadFileChunkLoadingRuntimeModule::new(**runtime_requirements).boxed(),
        )
      } else {
        compilation.add_runtime_module(
          chunk,
          RequireChunkLoadingRuntimeModule::new(**runtime_requirements).boxed(),
        );
      }
    }

    Ok(())
  }
}
