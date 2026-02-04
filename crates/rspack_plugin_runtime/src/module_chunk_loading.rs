use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationRuntimeRequirementInTree, Plugin,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{
  ExportRequireRuntimeModule, ModuleChunkLoadingRuntimeModule, is_enabled_for_chunk,
};

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleChunkLoadingPlugin;

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for ModuleChunkLoadingPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _additional_runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Import);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if is_enabled_for_chunk
    && compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements.insert(RuntimeGlobals::ASYNC_STARTUP);
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for ModuleChunkLoadingPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Import);
  if compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements_mut.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
  }

  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if !is_enabled_for_chunk {
    return Ok(None);
  }

  let has_chunk_loading_runtime_globals = RuntimeGlobals::ENSURE_CHUNK_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
    | RuntimeGlobals::ON_CHUNKS_LOADED
    | RuntimeGlobals::BASE_URI
    | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK;

  if runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
    runtime_modules_to_add.push((
      *chunk_ukey,
      ModuleChunkLoadingRuntimeModule::new(&compilation.runtime_template).boxed(),
    ));
  }

  if runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK) {
    runtime_modules_to_add.push((
      *chunk_ukey,
      ExportRequireRuntimeModule::new(&compilation.runtime_template).boxed(),
    ));
  }

  if !all_runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
    return Ok(None);
  }

  if all_runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK)
    || all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
  {
    runtime_requirements_mut
      .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_basic());
  }
  if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
    runtime_requirements_mut
      .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_with_loading());
    if all_runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS) {
      runtime_requirements_mut
        .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_with_prefetch());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS) {
      runtime_requirements_mut
        .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_with_preload());
    }
  }
  if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
    runtime_requirements_mut
      .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr());
  }
  if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST) {
    runtime_requirements_mut
      .extend(ModuleChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr_manifest());
  }

  Ok(None)
}

impl Plugin for ModuleChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "ModuleChunkLoadingPlugin"
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
