use super::*;

pub async fn optimize_modules_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let mut diagnostics = vec![];
  while matches!(
    plugin_driver
      .compilation_hooks
      .optimize_modules
      .call(compilation, &mut diagnostics)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeModules"))?,
    Some(true)
  ) {}
  compilation.extend_diagnostics(diagnostics);

  plugin_driver
    .compilation_hooks
    .after_optimize_modules
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterOptimizeModules"))
}
