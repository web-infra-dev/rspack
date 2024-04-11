use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation, CompilationRuntimeRequirementInTree,
  Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{is_enabled_for_chunk, JsonpChunkLoadingRuntimeModule};

#[plugin]
#[derive(Debug, Default)]
pub struct JsonpChunkLoadingPlugin;

#[plugin_hook(CompilationRuntimeRequirementInTree for JsonpChunkLoadingPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Jsonp);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);

  let mut has_jsonp_chunk_loading = false;
  for runtime_requirement in runtime_requirements.iter() {
    match runtime_requirement {
      RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
        has_jsonp_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::LOAD_SCRIPT);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
      }
      RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS if is_enabled_for_chunk => {
        has_jsonp_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::LOAD_SCRIPT);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
        runtime_requirements_mut.insert(RuntimeGlobals::MODULE_CACHE);
        runtime_requirements_mut.insert(RuntimeGlobals::HMR_MODULE_DATA);
        runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      }
      RuntimeGlobals::HMR_DOWNLOAD_MANIFEST if is_enabled_for_chunk => {
        has_jsonp_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME);
      }
      RuntimeGlobals::ON_CHUNKS_LOADED | RuntimeGlobals::BASE_URI if is_enabled_for_chunk => {
        has_jsonp_chunk_loading = true;
      }
      _ => {}
    }

    if has_jsonp_chunk_loading && is_enabled_for_chunk {
      runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      compilation
        .add_runtime_module(chunk_ukey, Box::<JsonpChunkLoadingRuntimeModule>::default())?;
    }
  }
  Ok(None)
}

impl Plugin for JsonpChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "JsonpChunkLoadingPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut rspack_core::ApplyContext>,
    _options: &mut rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
