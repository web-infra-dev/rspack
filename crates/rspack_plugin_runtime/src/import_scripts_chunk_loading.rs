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
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::ImportScripts);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if is_enabled_for_chunk
    && compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES);
    runtime_requirements.insert(RuntimeGlobals::ASYNC_STARTUP);
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ImportScriptsChunkLoadingPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::ImportScripts);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if !is_enabled_for_chunk {
    return Ok(None);
  }

  let has_chunk_loading_runtime_globals = RuntimeGlobals::ENSURE_CHUNK_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
    | RuntimeGlobals::BASE_URI
    | RuntimeGlobals::ON_CHUNKS_LOADED;

  if runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
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

  if !all_runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
    return Ok(None);
  }

  if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
    || all_runtime_requirements.contains(RuntimeGlobals::CHUNK_CALLBACK)
  {
    runtime_requirements_mut
      .extend(ImportScriptsChunkLoadingRuntimeModule::get_runtime_requirements_basic());
  }

  if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    runtime_requirements_mut
      .extend(ImportScriptsChunkLoadingRuntimeModule::get_runtime_requirements_with_loading());
  }

  if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
    runtime_requirements_mut
      .extend(ImportScriptsChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr());
  }

  if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST) {
    runtime_requirements_mut
      .extend(ImportScriptsChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr_manifest());
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
