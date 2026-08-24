use async_trait::async_trait;
use rspack_error::Result;
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::build_module_graph_pass;
use crate::{
  Compilation,
  compilation::{
    finish_make::finish_make_pass, finish_module_graph::finish_module_graph_pass,
    make::make_hook_pass, pass::PassExt,
  },
  legacy_cache::Cache,
  logger::Logger,
};

/// Composite pass for the entire Build Module Graph phase.
///
/// This phase includes multiple sub-passes:
/// - make hook
/// - build module graph
/// - finish make
/// - finish module graph
pub struct BuildModuleGraphPhasePass;

#[async_trait]
impl PassExt for BuildModuleGraphPhasePass {
  fn name(&self) -> &'static str {
    "build module graph"
  }

  fn incremental_passes(&self) -> crate::incremental::IncrementalPasses {
    crate::incremental::IncrementalPasses::BUILD_MODULE_GRAPH
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    restore_dependency_id_counter(compilation);
    cache.before_build_module_graph(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    let logger = compilation.get_logger("rspack.Compiler");
    // align with webpack, make hook include build_module_graph phase in webpack
    let start = logger.time("make hook");
    make_hook_pass(compilation, plugin_driver.clone()).await?;
    build_module_graph_pass(compilation).await?;
    logger.time_end(start);

    finish_make_pass(compilation, plugin_driver).await?;
    finish_module_graph_pass(compilation).await?;

    use crate::incremental::IncrementalPasses;
    if compilation
      .incremental
      .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
    {
      compilation
        .build_module_graph_artifact
        .module_graph
        .checkpoint();
    }
    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    if let Some(module_build_cache) = compilation.module_build_cache()
      && let Err(error) =
        module_build_cache.store_dependency_id_counter(get_current_dependency_id())
    {
      tracing::warn!("Storing the dependency id counter to the cache failed: {error}");
    }
    cache.after_build_module_graph(compilation).await;
  }
}

/// Cached modules keep the dependency ids of the run that produced them, so the
/// generator has to start above the highest id handed out so far. Only a cold
/// compiler run may move the counter, otherwise ids already in use would be
/// handed out twice.
fn restore_dependency_id_counter(compilation: &Compilation) {
  if get_current_dependency_id() != 0 {
    return;
  }
  let Some(module_build_cache) = compilation.module_build_cache() else {
    return;
  };
  match module_build_cache.restore_dependency_id_counter() {
    Ok(Some(counter)) => set_current_dependency_id(counter),
    Ok(None) => {}
    Err(error) => {
      tracing::warn!("Restoring the dependency id counter from the cache failed: {error}")
    }
  }
}
