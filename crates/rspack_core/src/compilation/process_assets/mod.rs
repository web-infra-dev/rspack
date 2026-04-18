use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

pub struct ProcessAssetsPass;

#[async_trait]
impl PassExt for ProcessAssetsPass {
  fn name(&self) -> &'static str {
    "process assets"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_process_assets(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    process_assets(compilation, plugin_driver).await
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_process_assets(compilation).await;
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
