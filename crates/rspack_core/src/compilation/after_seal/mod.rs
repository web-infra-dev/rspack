use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct AfterSealPass;

#[async_trait]
impl PassExt for AfterSealPass {
  fn name(&self) -> &'static str {
    "after seal"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    after_seal(compilation, plugin_driver).await?;
    Ok(())
  }
}

#[instrument("Compilation:after_seal", target=TRACING_BENCH_TARGET,skip_all)]
pub async fn after_seal(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  plugin_driver
    .compilation_hooks
    .after_seal
    .call(compilation)
    .await
}
