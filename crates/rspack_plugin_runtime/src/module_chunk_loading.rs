use async_trait::async_trait;
use rspack_core::{
  ChunkLoading, ChunkLoadingType, Plugin, PluginContext, PluginRuntimeRequirementsInTreeOutput,
  RuntimeGlobals, RuntimeModuleExt, RuntimeRequirementsInTreeArgs,
};

use crate::runtime_module::{
  is_enabled_for_chunk, ExportWebpackRequireRuntimeModule, ModuleChunkLoadingRuntimeModule,
};

#[derive(Debug)]
pub struct ModuleChunkLoadingPlugin;

#[async_trait]
impl Plugin for ModuleChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "ModuleChunkLoadingPlugin"
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Import);
    let is_enabled_for_chunk = is_enabled_for_chunk(chunk, &chunk_loading_value, compilation);
    let runtime_requirements = args.runtime_requirements;
    let runtime_requirements_mut = &mut args.runtime_requirements_mut;

    let mut has_chunk_loading = false;
    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
          has_chunk_loading = true;
          runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
        }
        RuntimeGlobals::EXTERNAL_INSTALL_CHUNK if is_enabled_for_chunk => {
          has_chunk_loading = true;
          compilation.add_runtime_module(chunk, ExportWebpackRequireRuntimeModule::new().boxed());
        }
        RuntimeGlobals::ON_CHUNKS_LOADED | RuntimeGlobals::BASE_URI if is_enabled_for_chunk => {
          has_chunk_loading = true;
        }
        _ => {}
      }
    }

    if has_chunk_loading && is_enabled_for_chunk {
      runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      compilation.add_runtime_module(chunk, Box::<ModuleChunkLoadingRuntimeModule>::default());
    }

    Ok(())
  }
}
