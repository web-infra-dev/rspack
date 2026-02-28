use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

pub struct OptimizeChunkModulesPass;

#[async_trait]
impl PassExt for OptimizeChunkModulesPass {
  fn name(&self) -> &'static str {
    "optimize chunk modules"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_optimize_chunk_modules(compilation).await;
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

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_optimize_chunk_modules(compilation).await;
  }
}
