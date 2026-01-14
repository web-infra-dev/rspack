use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn optimize_code_generation_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("optimize code generation");
    plugin_driver
      .compilation_hooks
      .optimize_code_generation
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeCodeGeneration"))?;
    logger.time_end(start);
    Ok(())
  }
}
