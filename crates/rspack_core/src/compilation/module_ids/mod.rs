use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn module_ids_pass(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("module ids");

    // Check if MODULE_IDS pass is disabled, and clear artifact if needed
    if !self
      .incremental
      .passes_enabled(IncrementalPasses::MODULE_IDS)
    {
      self.module_ids_artifact.clear();
    }

    let mut diagnostics = vec![];
    let mut module_ids_artifact = mem::take(&mut self.module_ids_artifact);
    plugin_driver
      .compilation_hooks
      .module_ids
      .call(self, &mut module_ids_artifact, &mut diagnostics)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
    self.module_ids_artifact = module_ids_artifact;
    self.extend_diagnostics(diagnostics);
    logger.time_end(start);
    Ok(())
  }
}
