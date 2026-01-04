use rspack_core::{
  ChunkLoading, ChunkUkey, Compilation, CompilationRuntimeRequirementInTree, Plugin,
  RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::runtime_module::{
  StartupChunkDependenciesRuntimeModule, StartupEntrypointRuntimeModule, is_enabled_for_chunk,
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

#[plugin_hook(CompilationRuntimeRequirementInTree for StartupChunkDependenciesPlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  let mut async_chunk_loading = self.async_chunk_loading;
  if runtime_requirements.contains(RuntimeGlobals::ASYNC_FEDERATION_STARTUP) {
    async_chunk_loading = true;
  }

  if is_enabled_for_chunk
    && runtime_requirements.contains(RuntimeGlobals::STARTUP_CHUNK_DEPENDENCIES)
  {
    runtime_requirements_mut.insert(RuntimeGlobals::STARTUP);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupChunkDependenciesRuntimeModule::new(
        &compilation.runtime_template,
        async_chunk_loading,
      )
      .boxed(),
    )?;
  }

  if is_enabled_for_chunk && runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupEntrypointRuntimeModule::new(&compilation.runtime_template, async_chunk_loading)
        .boxed(),
    )?;
  }

  Ok(None)
}

impl Plugin for StartupChunkDependenciesPlugin {
  fn name(&self) -> &'static str {
    "StartupChunkDependenciesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}
