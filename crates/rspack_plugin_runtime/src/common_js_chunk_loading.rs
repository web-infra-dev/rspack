use rspack_core::{
  ChunkLoading, ChunkLoadingType, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationRuntimeRequirementInTree, Plugin,
  RuntimeGlobals, RuntimeModule, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{
  ReadFileChunkLoadingRuntimeModule, RequireChunkLoadingRuntimeModule, is_enabled_for_chunk,
};

#[plugin]
#[derive(Debug)]
pub struct CommonJsChunkLoadingPlugin {
  async_chunk_loading: bool,
}

impl CommonJsChunkLoadingPlugin {
  pub fn new(async_chunk_loading: bool) -> Self {
    Self::new_inner(async_chunk_loading)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for CommonJsChunkLoadingPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  _runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  let chunk_loading_value = if self.async_chunk_loading {
    ChunkLoading::Enable(ChunkLoadingType::AsyncNode)
  } else {
    ChunkLoading::Enable(ChunkLoadingType::Require)
  };
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);
  if is_enabled_for_chunk
    && compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES);
    if self.async_chunk_loading {
      runtime_requirements.insert(RuntimeGlobals::ASYNC_STARTUP);
    }
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for CommonJsChunkLoadingPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  let chunk_loading_value = if self.async_chunk_loading {
    ChunkLoading::Enable(ChunkLoadingType::AsyncNode)
  } else {
    ChunkLoading::Enable(ChunkLoadingType::Require)
  };
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &chunk_loading_value, compilation);

  if !is_enabled_for_chunk {
    return Ok(None);
  }

  let has_chunk_loading_runtime_globals = RuntimeGlobals::ENSURE_CHUNK_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS
    | RuntimeGlobals::HMR_DOWNLOAD_MANIFEST
    | RuntimeGlobals::ON_CHUNKS_LOADED
    | RuntimeGlobals::EXTERNAL_INSTALL_CHUNK
    | RuntimeGlobals::BASE_URI;

  if runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
    if self.async_chunk_loading {
      runtime_modules_to_add.push((
        *chunk_ukey,
        ReadFileChunkLoadingRuntimeModule::new(&compilation.runtime_template).boxed(),
      ));
    } else {
      runtime_modules_to_add.push((
        *chunk_ukey,
        RequireChunkLoadingRuntimeModule::new(&compilation.runtime_template).boxed(),
      ));
    }
  }

  if !all_runtime_requirements.intersects(has_chunk_loading_runtime_globals) {
    return Ok(None);
  }

  if self.async_chunk_loading {
    if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      | all_runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK)
    {
      runtime_requirements_mut
        .extend(ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_basic());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
      runtime_requirements_mut
        .extend(ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_with_loading());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
      runtime_requirements_mut
        .extend(ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST) {
      runtime_requirements_mut
        .extend(ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr_manifest());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
      runtime_requirements_mut
        .extend(ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_with_on_chunk_load());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK) {
      runtime_requirements_mut.extend(
        ReadFileChunkLoadingRuntimeModule::get_runtime_requirements_with_external_install_chunk(),
      );
    }
  } else {
    if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
      | all_runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK)
    {
      runtime_requirements_mut
        .extend(RequireChunkLoadingRuntimeModule::get_runtime_requirements_basic());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS) {
      runtime_requirements_mut
        .extend(RequireChunkLoadingRuntimeModule::get_runtime_requirements_with_loading());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS) {
      runtime_requirements_mut
        .extend(RequireChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_MANIFEST) {
      runtime_requirements_mut
        .extend(RequireChunkLoadingRuntimeModule::get_runtime_requirements_with_hmr_manifest());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::ON_CHUNKS_LOADED) {
      runtime_requirements_mut
        .extend(RequireChunkLoadingRuntimeModule::get_runtime_requirements_with_on_chunk_load());
    }
    if all_runtime_requirements.contains(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK) {
      runtime_requirements_mut.extend(
        RequireChunkLoadingRuntimeModule::get_runtime_requirements_with_external_install_chunk(),
      );
    }
  }

  Ok(None)
}

impl Plugin for CommonJsChunkLoadingPlugin {
  fn name(&self) -> &'static str {
    "CommonJsChunkLoadingPlugin"
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
