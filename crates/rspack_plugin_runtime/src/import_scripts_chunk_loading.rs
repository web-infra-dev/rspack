use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationRuntimeRequirementInTree, Plugin,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{ImportScriptsChunkLoadingRuntimeModule, is_enabled_for_chunk};

#[plugin]
#[derive(Debug, Default)]
pub struct ImportScriptsChunkLoadingPlugin;

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ImportScriptsChunkLoadingPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::ImportScripts);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if is_enabled_for_chunk
    && compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES);
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ImportScriptsChunkLoadingPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::ImportScripts);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);

  let mut has_chunk_loading = false;
  for runtime_requirement in runtime_requirements.iter() {
    match runtime_requirement {
      RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
        has_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
      }
      RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS if is_enabled_for_chunk => {
        has_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME);
        runtime_requirements_mut.insert(RuntimeGlobals::MODULE_CACHE);
        runtime_requirements_mut.insert(RuntimeGlobals::HMR_MODULE_DATA);
        runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      }
      RuntimeGlobals::HMR_DOWNLOAD_MANIFEST if is_enabled_for_chunk => {
        has_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME);
      }
      RuntimeGlobals::BASE_URI | RuntimeGlobals::ON_CHUNKS_LOADED if is_enabled_for_chunk => {
        has_chunk_loading = true;
      }
      _ => {}
    }

    if has_chunk_loading && is_enabled_for_chunk {
      runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
      runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
      let with_create_script_url = compilation.options.output.trusted_types.is_some();
      if with_create_script_url {
        runtime_requirements_mut.insert(RuntimeGlobals::CREATE_SCRIPT_URL);
      }
      runtime_modules_to_add.push((
        *chunk_ukey,
        ImportScriptsChunkLoadingRuntimeModule::new(
          &compilation.runtime_template,
          with_create_script_url,
        )
        .boxed(),
      ));
    }
  }
  Ok(None)
}

impl Plugin for ImportScriptsChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "ImportScriptsChunkLoadingPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
