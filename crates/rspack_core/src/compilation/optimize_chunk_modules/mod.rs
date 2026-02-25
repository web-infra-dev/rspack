use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

pub struct OptimizeChunkModulesPass;

#[async_trait]
impl PassExt for OptimizeChunkModulesPass {
  fn name(&self) -> &'static str {
    "optimize chunk modules"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_optimize_chunk_modules(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let mut build_chunk_graph_artifact =
      std::mem::take(&mut compilation.build_chunk_graph_artifact);
    let mut build_module_graph_artifact = compilation.build_module_graph_artifact.steal();
    let mut async_modules_artifact = compilation.async_modules_artifact.steal();
    let mut exports_info_artifact = compilation.exports_info_artifact.steal();
    let mut imported_by_defer_modules_artifact =
      compilation.imported_by_defer_modules_artifact.steal();
    let mut diagnostics = vec![];

    while matches!(
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .optimize_chunk_modules
        .call(
          compilation,
          &mut build_chunk_graph_artifact,
          &mut build_module_graph_artifact,
          &mut async_modules_artifact,
          &mut exports_info_artifact,
          &mut imported_by_defer_modules_artifact,
          &mut diagnostics,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))?,
      Some(true)
    ) {}

    compilation.build_chunk_graph_artifact = build_chunk_graph_artifact;
    compilation.build_module_graph_artifact = build_module_graph_artifact.into();
    compilation.async_modules_artifact = async_modules_artifact.into();
    compilation.exports_info_artifact = exports_info_artifact.into();
    compilation.imported_by_defer_modules_artifact = imported_by_defer_modules_artifact.into();
    compilation.extend_diagnostics(diagnostics);

    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_optimize_chunk_modules(compilation).await;
  }
}
