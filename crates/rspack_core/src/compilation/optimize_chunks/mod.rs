use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct OptimizeChunksPass;

#[async_trait]
impl PassExt for OptimizeChunksPass {
  fn name(&self) -> &'static str {
    "optimize chunks"
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    while matches!(
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .optimize_chunks
        .call(compilation)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunks"))?,
      Some(true)
    ) {}

    Ok(())
  }
}
