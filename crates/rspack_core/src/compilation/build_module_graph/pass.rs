use async_trait::async_trait;
use rspack_error::Result;

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

async fn abort_module_executor(compilation: &mut Compilation) -> Result<()> {
  if let Some(module_executor) = &mut compilation.module_executor {
    let mut module_executor = std::mem::take(module_executor);
    let result = module_executor.abort_rebuild_module_graph().await;
    compilation.module_executor = Some(module_executor);
    result
  } else {
    Ok(())
  }
}

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
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    let logger = compilation.get_logger("rspack.Compiler");
    // align with webpack, make hook include build_module_graph phase in webpack
    let start = logger.time("make hook");
    make_hook_pass(compilation, plugin_driver.clone()).await?;
    if let Err(error) = build_module_graph_pass(compilation).await {
      if let Err(abort_error) = abort_module_executor(compilation).await {
        tracing::error!(
          ?abort_error,
          "failed to abort module executor after make failure"
        );
      }
      return Err(error);
    }
    logger.time_end(start);

    if let Err(error) = finish_make_pass(compilation, plugin_driver).await {
      if let Err(abort_error) = abort_module_executor(compilation).await {
        tracing::error!(
          ?abort_error,
          "failed to abort module executor after finish make failure"
        );
      }
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

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) -> Result<()> {
    if let Some(module_build_cache) = compilation.module_build_cache.clone() {
      module_build_cache
        .store_pending(
          &mut compilation.build_module_graph_artifact,
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
