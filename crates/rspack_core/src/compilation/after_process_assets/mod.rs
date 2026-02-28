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
    compilation.after_process_assets(plugin_driver).await
  }
}

impl Compilation {
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
