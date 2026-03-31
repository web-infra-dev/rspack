use async_trait::async_trait;
use rspack_error::Result;

use super::build_module_graph_pass;
use crate::{
  Compilation,
  cache::Cache,
  compilation::{finish_make::finish_make_pass, make::make_hook_pass, pass::PassExt},
  logger::Logger,
};

/// Composite pass for the entire Build Module Graph phase.
///
/// This phase includes multiple sub-passes:
/// - make hook
/// - build module graph
/// - finish make
pub struct BuildModuleGraphPhasePass;

#[async_trait]
impl PassExt for BuildModuleGraphPhasePass {
  fn name(&self) -> &'static str {
    "build module graph"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_build_module_graph(compilation).await;
  }

  async fn run_pass(&self, _compilation: &mut Compilation) -> Result<()> {
    // This method is not used; we override run_pass_with_cache instead
    unreachable!("BuildModuleGraphPhasePass uses run_pass_with_cache")
  }

  async fn run_pass_with_cache(
    &self,
    compilation: &mut Compilation,
    _cache: &mut dyn Cache,
  ) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();
    let logger = compilation.get_logger("rspack.Compiler");

    // webpack's `make hook` timing includes building the module graph.
    let start = logger.time("make hook");
    make_hook_pass(compilation, plugin_driver.clone()).await?;
    build_module_graph_pass(compilation).await?;
    logger.time_end(start);

    let start = logger.time("finish make hook");
    finish_make_pass(compilation, plugin_driver.clone()).await?;
    logger.time_end(start);

    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_build_module_graph(compilation).await;
  }
}
