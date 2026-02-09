use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

pub struct OptimizeDependenciesPass;

#[async_trait]
impl PassExt for OptimizeDependenciesPass {
  fn name(&self) -> &'static str {
    "optimize dependencies"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_optimize_dependencies(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let mut side_effects_optimize_artifact = compilation.side_effects_optimize_artifact.steal();
    let mut build_module_graph_artifact = compilation.build_module_graph_artifact.steal();
    while matches!(
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .optimize_dependencies
        .call(
          compilation,
          &mut side_effects_optimize_artifact,
          &mut build_module_graph_artifact,
          &mut diagnostics
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeDependencies"))?,
      Some(true)
    ) {}
    compilation.side_effects_optimize_artifact = side_effects_optimize_artifact.into();
    compilation.build_module_graph_artifact = build_module_graph_artifact.into();
    compilation.extend_diagnostics(diagnostics);

    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_optimize_dependencies(compilation).await;
  }
}
