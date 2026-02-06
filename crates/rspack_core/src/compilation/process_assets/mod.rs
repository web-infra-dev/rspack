use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub(super) struct ProcessAssetsPass;

#[async_trait]
impl PassExt for ProcessAssetsPass {
  fn name(&self) -> &'static str {
    "process assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    compilation.process_assets(plugin_driver).await
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
}
