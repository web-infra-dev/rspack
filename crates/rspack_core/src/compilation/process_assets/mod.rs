use async_trait::async_trait;

use super::*;
use crate::{compilation::pass::PassExt, logger::Logger};

pub struct ProcessAssetsPass;

#[async_trait]
impl PassExt for ProcessAssetsPass {
  fn name(&self) -> &'static str {
    "process assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    compilation.process_assets(plugin_driver.clone()).await?;

    let logger = compilation.get_logger("rspack.Compilation");
    let start = logger.time("after process assets");
    compilation.after_process_assets(plugin_driver).await?;
    logger.time_end(start);
    Ok(())
  }
}

impl Compilation {
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
