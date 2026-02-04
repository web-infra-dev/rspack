use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct OptimizeChunkModulesPass;

#[async_trait]
impl PassExt for OptimizeChunkModulesPass {
  fn name(&self) -> &'static str {
    "optimize chunk modules"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_chunk_modules
      .call(compilation)
      .await
      .map(|_| ())
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))?;

    Ok(())
  }
}
