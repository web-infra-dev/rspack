use async_trait::async_trait;
use rspack_core::{
  ApplyContext, ChunkLoading, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationRuntimeRequirementInTree, CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
  RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{
  is_enabled_for_chunk, StartupChunkDependenciesRuntimeModule, StartupEntrypointRuntimeModule,
};

#[plugin]
#[derive(Debug)]
pub struct StartupChunkDependenciesPlugin {
  chunk_loading: ChunkLoading,
  async_chunk_loading: bool,
}

impl StartupChunkDependenciesPlugin {
  pub fn new(chunk_loading: ChunkLoading, async_chunk_loading: bool) -> Self {
    Self::new_inner(chunk_loading, async_chunk_loading)
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for StartupChunkDependenciesPlugin)]
fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  if compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey)
    && is_enabled_for_chunk
  {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupChunkDependenciesRuntimeModule::new(self.async_chunk_loading).boxed(),
    )?;
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInTree for StartupChunkDependenciesPlugin)]
fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);

  if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) && is_enabled_for_chunk {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupEntrypointRuntimeModule::new(self.async_chunk_loading).boxed(),
    )?;
  }

  Ok(None)
}

#[async_trait]
impl Plugin for StartupChunkDependenciesPlugin {
  fn name(&self) -> &'static str {
    "StartupChunkDependenciesPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .context
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
