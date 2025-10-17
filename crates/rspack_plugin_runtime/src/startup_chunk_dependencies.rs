use rspack_core::{
  ChunkGraph, ChunkLoading, ChunkUkey, Compilation, CompilationAdditionalTreeRuntimeRequirements,
  CompilationRuntimeRequirementInTree, Plugin, RuntimeGlobals, RuntimeModuleExt, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use crate::{
  chunk_needs_mf_async_startup,
  runtime_module::{
    StartupChunkDependenciesRuntimeModule, StartupEntrypointRuntimeModule, is_enabled_for_chunk,
  },
};

#[plugin]
#[derive(Debug)]
pub struct StartupChunkDependenciesPlugin {
  chunk_loading: ChunkLoading,
  async_chunk_loading: bool,
}

impl StartupChunkDependenciesPlugin {
  #[inline]
  fn should_enable_async_startup(
    mf_async_startup_enabled: bool,
    has_federation_runtime: bool,
    has_federation_handlers: bool,
    has_remote_modules: bool,
    has_consume_modules: bool,
  ) -> bool {
    if !mf_async_startup_enabled {
      return false;
    }

    if has_federation_runtime && has_federation_handlers {
      return true;
    }

    if has_federation_runtime {
      return true;
    }

    has_remote_modules || has_consume_modules
  }

  pub fn new(chunk_loading: ChunkLoading, async_chunk_loading: bool) -> Self {
    Self::new_inner(chunk_loading, async_chunk_loading)
  }

  fn is_async_enabled(&self, compilation: &Compilation, chunk_ukey: &ChunkUkey) -> bool {
    if chunk_needs_mf_async_startup(compilation, chunk_ukey) {
      return true;
    }

    if self.async_chunk_loading && compilation.options.experiments.mf_async_startup {
      return true;
    }

    let runtime_requirements = ChunkGraph::get_chunk_runtime_requirements(compilation, chunk_ukey);
    let has_federation_runtime = runtime_requirements.contains(RuntimeGlobals::INITIALIZE_SHARING)
      || runtime_requirements.contains(RuntimeGlobals::SHARE_SCOPE_MAP)
      || runtime_requirements.contains(RuntimeGlobals::CURRENT_REMOTE_GET_SCOPE);
    let has_federation_handlers =
      runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);

    let module_graph = compilation.get_module_graph();
    let has_remote_modules = !compilation
      .chunk_graph
      .get_chunk_modules_by_source_type(chunk_ukey, SourceType::Remote, &module_graph)
      .is_empty();
    let has_consume_modules = !compilation
      .chunk_graph
      .get_chunk_modules_by_source_type(chunk_ukey, SourceType::ConsumeShared, &module_graph)
      .is_empty();

    Self::should_enable_async_startup(
      compilation.options.experiments.mf_async_startup,
      has_federation_runtime,
      has_federation_handlers,
      has_remote_modules,
      has_consume_modules,
    )
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for StartupChunkDependenciesPlugin)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let is_enabled_for_chunk = is_enabled_for_chunk(chunk_ukey, &self.chunk_loading, compilation);
  let has_entry_deps = compilation
    .chunk_graph
    .has_chunk_entry_dependent_chunks(chunk_ukey, &compilation.chunk_group_by_ukey);
  let async_enabled = self.is_async_enabled(compilation, chunk_ukey);

  if (has_entry_deps && is_enabled_for_chunk) || async_enabled {
    runtime_requirements.insert(RuntimeGlobals::STARTUP);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupChunkDependenciesRuntimeModule::new(async_enabled).boxed(),
    )?;
  }
  Ok(())
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
  let async_enabled = self.is_async_enabled(compilation, chunk_ukey);

  if runtime_requirements.contains(RuntimeGlobals::STARTUP_ENTRYPOINT)
    && (is_enabled_for_chunk || async_enabled)
  {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK);
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES);
    compilation.add_runtime_module(
      chunk_ukey,
      StartupEntrypointRuntimeModule::new(async_enabled).boxed(),
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
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::StartupChunkDependenciesPlugin;

  #[test]
  fn respects_experiment_flag_before_enabling_async_startup() {
    assert!(
      !StartupChunkDependenciesPlugin::should_enable_async_startup(false, true, true, false, false)
    );
  }

  #[test]
  fn enables_when_flag_and_runtime_requirements_present() {
    assert!(StartupChunkDependenciesPlugin::should_enable_async_startup(
      true, true, true, false, false
    ));
  }

  #[test]
  fn enables_when_flag_and_remote_or_consume_modules_present() {
    assert!(StartupChunkDependenciesPlugin::should_enable_async_startup(
      true, false, false, true, false
    ));
    assert!(StartupChunkDependenciesPlugin::should_enable_async_startup(
      true, false, false, false, true
    ));
  }

  #[test]
  fn remains_disabled_when_no_async_signals() {
    assert!(
      !StartupChunkDependenciesPlugin::should_enable_async_startup(
        true, false, false, false, false
      )
    );
  }
}
