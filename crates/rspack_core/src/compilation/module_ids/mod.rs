use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

pub struct ModuleIdsPass;

#[async_trait]
impl PassExt for ModuleIdsPass {
  fn name(&self) -> &'static str {
    "module ids"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_module_ids(compilation).await;
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    // Check if MODULE_IDS pass is disabled, and clear artifact if needed
    if !compilation
      .incremental
      .passes_enabled(IncrementalPasses::MODULE_IDS)
    {
      compilation.module_ids_artifact.clear();
    }

    let mut diagnostics = vec![];
    let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .module_ids
      .call(compilation, &mut module_ids_artifact, &mut diagnostics)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
    compilation.module_ids_artifact = module_ids_artifact;
    compilation.extend_diagnostics(diagnostics);
    Ok(())
  }

  async fn after_pass(&self, compilation: &Compilation, cache: &mut dyn Cache) {
    cache.after_module_ids(compilation).await;
  }
}
