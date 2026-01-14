use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn after_seal_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("after seal");
    self.after_seal(plugin_driver).await?;
    logger.time_end(start);
    Ok(())
  }

  #[instrument("Compilation:after_seal", target=TRACING_BENCH_TARGET,skip_all)]
  async fn after_seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.compilation_hooks.after_seal.call(self).await
  }
}
