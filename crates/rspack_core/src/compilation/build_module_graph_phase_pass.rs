use async_trait::async_trait;
use rspack_error::Result;

use crate::{
  Compilation,
  cache::Cache,
  compilation::{
    build_module_graph::build_module_graph_pass, finish_make::finish_make_pass,
    finish_module_graph::finish_module_graph_pass, finish_modules::finish_modules_pass,
    make::make_hook_pass, pass::PassExt,
  },
};

/// Composite pass for the entire Build Module Graph phase.
///
/// This phase includes multiple sub-passes:
/// - make hook
/// - build module graph
/// - finish make
/// - finish module graph
/// - finish modules (has its own cache interaction)
///
/// This pass uses `run_pass_with_cache` because finish_modules needs cache access mid-pass.
pub struct BuildModuleGraphPhasePass;

#[async_trait]
impl PassExt for BuildModuleGraphPhasePass {
  fn name(&self) -> &'static str {
    "build module graph"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache
      .before_build_module_graph(
        &mut compilation.build_module_graph_artifact,
        &compilation.incremental,
      )
      .await;
  }

  async fn run_pass(&self, _compilation: &mut Compilation) -> Result<()> {
    // This method is not used; we override run_pass_with_cache instead
    unreachable!("BuildModuleGraphPhasePass uses run_pass_with_cache")
  }

  async fn run_pass_with_cache(
    &self,
    compilation: &mut Compilation,
    cache: &mut dyn Cache,
  ) -> Result<()> {
    let plugin_driver = compilation.plugin_driver.clone();

    // Sub-phase: make hook
    make_hook_pass(compilation, plugin_driver.clone()).await?;

    // Sub-phase: build module graph
    build_module_graph_pass(compilation).await?;

    // Sub-phase: finish make
    finish_make_pass(compilation, plugin_driver.clone()).await?;

    // Sub-phase: finish module graph
    finish_module_graph_pass(compilation).await?;

    // Sub-phase: finish_modules (with inline cache handling)
    cache.before_finish_modules(compilation).await;
    finish_modules_pass(compilation).await?;
    cache.after_finish_modules(compilation).await;
    // Add checkpoint if incremental build is enabled
    use crate::incremental::IncrementalPasses;
    if compilation
      .incremental
      .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
    {
      compilation
        .build_module_graph_artifact
        .module_graph
        .checkpoint();
    }
    Ok(())
  }

  async fn after_pass(&self, compilation: &Compilation, cache: &mut dyn Cache) {
    cache
      .after_build_module_graph(&compilation.build_module_graph_artifact)
      .await;
  }
}
