use std::sync::atomic::Ordering;

use async_trait::async_trait;
use rspack_error::Result;

use super::{build_module_graph_pass, module_build_cache::ModuleBuildCache};
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
    cache.before_build_module_graph(compilation).await;
    // Initialize after recovery and before make can build modules.
    // Rebuilds publish modules but handle invalidation without cache restoration.
    compilation.build_module_graph_artifact.module_build_cache =
      compilation.options.experiments.new_cache.module.then(|| {
        ModuleBuildCache::new(
          compilation.cache.facade("Compilation/modules"),
          !compilation.is_rebuild,
        )
      });
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let _loader_cache_guard = compilation
      .loader_cache()
      .upgrade()
      .map(|cache| cache.begin_build());
    let result = run_build_module_graph_phase(compilation).await;
    compilation.in_finish_make.store(false, Ordering::Release);
    if result.is_err()
      && let Some(executor) = compilation.module_executor.take()
    {
      executor.abort().await;
    }
    result
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) -> Result<()> {
    if let Some(module_build_cache) = &compilation.build_module_graph_artifact.module_build_cache {
      module_build_cache
        .store_pending(
          &compilation.build_module_graph_artifact,
          &compilation.file_system_info,
        )
        .await?;
    }
    for (_, module) in compilation.get_module_graph().modules() {
      module.freeze_build_info();
    }
    cache.after_build_module_graph(compilation).await;
    Ok(())
  }
}

async fn run_build_module_graph_phase(compilation: &mut Compilation) -> Result<()> {
  let plugin_driver = compilation.plugin_driver.clone();
  let logger = compilation.get_logger("rspack.Compiler");
  // align with webpack, make hook include build_module_graph phase in webpack
  let start = logger.time("make hook");
  make_hook_pass(compilation, plugin_driver.clone()).await?;
  if let Some(error) = compilation.build_module_graph_error.take() {
    return Err(error);
  }
  build_module_graph_pass(compilation).await?;
  logger.time_end(start);

  finish_make_pass(compilation, plugin_driver).await?;
  if let Some(error) = compilation.build_module_graph_error.take() {
    return Err(error);
  }
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
