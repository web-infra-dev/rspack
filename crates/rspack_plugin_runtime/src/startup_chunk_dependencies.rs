use async_trait::async_trait;
use rspack_core::{
  is_enabled_for_chunk, AdditionalChunkRuntimeRequirementsArgs, ChunkLoading, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, RuntimeGlobals, RuntimeModuleExt,
};
use rspack_error::Result;

use crate::runtime_module::{
  StartupChunkDependenciesRuntimeModule, StartupEntrypointRuntimeModule,
};

#[derive(Debug)]
pub struct StartupChunkDependenciesPlugin {
  chunk_loading: ChunkLoading,
  async_chunk_loading: bool,
}

impl StartupChunkDependenciesPlugin {
  pub fn new(chunk_loading: ChunkLoading, async_chunk_loading: bool) -> Self {
    Self {
      chunk_loading,
      async_chunk_loading,
    }
  }
}

#[async_trait]
impl Plugin for StartupChunkDependenciesPlugin {
  fn name(&self) -> &'static str {
    "StartupChunkDependenciesPlugin"
  }

  fn apply(&self, _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>) -> Result<()> {
    Ok(())
  }

  fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation: &mut &mut rspack_core::Compilation = &mut args.compilation;
    let is_enabled_for_chunk = is_enabled_for_chunk(args.chunk, &self.chunk_loading, compilation);
    let runtime_requirements = &mut args.runtime_requirements;
    if compilation
      .chunk_graph
      .has_chunk_entry_dependent_chunks(args.chunk, &compilation.chunk_group_by_ukey)
      && is_enabled_for_chunk
    {
      runtime_requirements.insert(RuntimeGlobals::STARTUP);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
      compilation.add_runtime_module(
        args.chunk,
        StartupChunkDependenciesRuntimeModule::new(self.async_chunk_loading).boxed(),
      );
    }
    Ok(())
  }

  fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let is_enabled_for_chunk = is_enabled_for_chunk(args.chunk, &self.chunk_loading, compilation);
    let runtime_requirements = &mut args.runtime_requirements;

    if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT) && is_enabled_for_chunk {
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
      runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
      compilation.add_runtime_module(
        args.chunk,
        StartupEntrypointRuntimeModule::new(self.async_chunk_loading).boxed(),
      );
    }

    Ok(())
  }
}
