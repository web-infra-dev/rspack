use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub(super) struct OptimizeTreePass;

#[async_trait]
impl PassExt for OptimizeTreePass {
  fn name(&self) -> &'static str {
    "optimize tree"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_tree
      .call(compilation)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeTree"))?;

    Ok(())
  }
}
