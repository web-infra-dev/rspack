use super::*;
use crate::logger::Logger;

impl Compilation {
  pub async fn process_assets_pass(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let start = logger.time("process assets");
    self.process_assets(plugin_driver.clone()).await?;
    logger.time_end(start);

    let start = logger.time("after process assets");
    self.after_process_assets(plugin_driver).await?;
    logger.time_end(start);
    Ok(())
  }

  #[instrument("Compilation:process_assets",target=TRACING_BENCH_TARGET, skip_all)]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .compilation_hooks
      .process_assets
      .call(self)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"))
  }

  #[instrument("Compilation:after_process_assets", skip_all)]
  async fn after_process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let mut diagnostics: Vec<Diagnostic> = vec![];

    let res = plugin_driver
      .compilation_hooks
      .after_process_assets
      .call(self, &mut diagnostics)
      .await;

    self.extend_diagnostics(diagnostics);
    res
  }
}
