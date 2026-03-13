use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct AfterProcessAssetsPass;

#[async_trait]
impl PassExt for AfterProcessAssetsPass {
  fn name(&self) -> &'static str {
    "after process assets"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    after_process_assets(compilation, plugin_driver).await
  }
}

#[instrument("Compilation:after_process_assets", skip_all)]
pub async fn after_process_assets(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let mut diagnostics: Vec<Diagnostic> = vec![];

  let res = plugin_driver
    .compilation_hooks
    .after_process_assets
    .call(compilation, &mut diagnostics)
    .await;

  compilation.extend_diagnostics(diagnostics);
  res
}
