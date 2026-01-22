use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationRuntimeRequirementInTree, Plugin,
  PublicPath, RuntimeGlobals, RuntimeModule, RuntimeModuleExt,
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
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
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
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  let chunk_loading_value = ChunkLoading::Enable(ChunkLoadingType::Import);
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);

  let mut has_chunk_loading = false;

  for runtime_requirement in runtime_requirements.iter() {
    match runtime_requirement {
      RuntimeGlobals::ENSURE_CHUNK_HANDLERS if is_enabled_for_chunk => {
        has_chunk_loading = true;
        if !matches!(compilation.options.output.public_path, PublicPath::Auto) {
          runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        }
        runtime_requirements_mut.insert(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME);
      }
      RuntimeGlobals::EXTERNAL_INSTALL_CHUNK if is_enabled_for_chunk => {
        has_chunk_loading = true;

        runtime_modules_to_add.push((
          *chunk_ukey,
          ExportRequireRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }

      RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS if is_enabled_for_chunk => {
        has_chunk_loading = true;
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
        runtime_requirements_mut.insert(RuntimeGlobals::LOAD_SCRIPT);
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
      RuntimeGlobals::BASE_URI
      | RuntimeGlobals::ON_CHUNKS_LOADED
      | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
      | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
        if is_enabled_for_chunk =>
      {
        has_chunk_loading = true;
      }
      RuntimeGlobals::PREFETCH_CHUNK_HANDLERS | RuntimeGlobals::PRELOAD_CHUNK_HANDLERS
        if has_chunk_loading =>
      {
        runtime_requirements_mut.insert(RuntimeGlobals::PUBLIC_PATH);
      }
      _ => {}
    }
  }

  if has_chunk_loading && is_enabled_for_chunk {
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY);
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
    runtime_modules_to_add.push((
      *chunk_ukey,
      ModuleChunkLoadingRuntimeModule::new(&compilation.runtime_template).boxed(),
    ));
  }

  if compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements_mut.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
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
