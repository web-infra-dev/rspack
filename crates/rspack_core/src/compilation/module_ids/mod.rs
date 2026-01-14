use super::*;
use crate::logger::Logger;

pub async fn module_ids_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("module ids");

  // Check if MODULE_IDS pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULE_IDS)
  {
    compilation.module_ids_artifact.clear();
  }

  let mut diagnostics = vec![];
  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);
  plugin_driver
    .compilation_hooks
    .module_ids
    .call(compilation, &mut module_ids_artifact, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
  compilation.module_ids_artifact = module_ids_artifact;
  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}
