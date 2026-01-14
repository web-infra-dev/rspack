use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn optimize_dependencies_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("optimize dependencies");
    // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let mut side_effects_optimize_artifact = self.side_effects_optimize_artifact.take();
    let mut build_module_graph_artifact = self.build_module_graph_artifact.take();
    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_dependencies
        .call(
          self,
          &mut side_effects_optimize_artifact,
          &mut build_module_graph_artifact,
          &mut diagnostics
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeDependencies"))?,
      Some(true)
    ) {}
    self
      .side_effects_optimize_artifact
      .replace(side_effects_optimize_artifact);
    self
      .build_module_graph_artifact
      .replace(build_module_graph_artifact);
    self.extend_diagnostics(diagnostics);

    logger.time_end(start);
    Ok(())
  }
}
