use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub(super) struct OptimizeModulesPass;

#[async_trait]
impl PassExt for OptimizeModulesPass {
  fn name(&self) -> &'static str {
    "optimize modules"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let mut diagnostics = vec![];
    while matches!(
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .optimize_modules
        .call(compilation, &mut diagnostics)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeModules"))?,
      Some(true)
    ) {}
    compilation.extend_diagnostics(diagnostics);

    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .after_optimize_modules
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterOptimizeModules"))?;

    Ok(())
  }
}
