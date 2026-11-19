use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

pub struct OptimizeChunkModulesPass;

#[async_trait]
impl PassExt for OptimizeChunkModulesPass {
  fn name(&self) -> &'static str {
    "optimize chunk modules"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::OPTIMIZE_CHUNK_MODULES
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    let previous_async_startup = std::mem::take(&mut compilation.async_modules_artifact.startup);
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .optimize_chunk_modules
      .call(compilation)
      .await
      .map(|_| ())
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeChunkModules"))?;

    if let Some(mut mutations) = compilation.incremental.mutations_write() {
      let module_graph = compilation.get_module_graph();
      for module in
        previous_async_startup.symmetric_difference(&compilation.async_modules_artifact.startup)
      {
        if module_graph.module_by_identifier(module).is_some() {
          mutations.add(Mutation::ModuleSetAsync { module: *module });
        }
      }
    }

    Ok(())
  }
}
