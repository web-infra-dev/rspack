use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct ProcessAssetsPass;

#[async_trait]
impl PassExt for ProcessAssetsPass {
  fn name(&self) -> &'static str {
    "process assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    process_assets(compilation, plugin_driver).await
  }
}

#[instrument("Compilation:process_assets",target=TRACING_BENCH_TARGET, skip_all)]
pub async fn process_assets(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver
    .compilation_hooks
    .process_assets
    .call(compilation)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.processAssets"))
}
