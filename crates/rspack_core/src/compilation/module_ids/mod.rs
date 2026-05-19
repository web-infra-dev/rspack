use async_trait::async_trait;

use super::*;
use crate::{cache::Cache, compilation::pass::PassExt};

/// Collects module identifiers that need ID assignment.
/// A module needs an ID if:
/// - It doesn't already have one assigned
/// - It needs an ID (need_id() returns true)
/// - It's part of at least one chunk
fn get_modules_needing_ids(
  compilation: &Compilation,
  module_ids_artifact: &ModuleIdsArtifact,
) -> IdentifierSet {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  compilation
    .get_module_graph()
    .modules()
    .map(|(_, module)| module)
    .filter(|m| {
      m.need_id()
        && ChunkGraph::get_module_id(module_ids_artifact, m.identifier()).is_none()
        && chunk_graph.get_number_of_module_chunks(m.identifier()) != 0
    })
    .map(|m| m.identifier())
    .collect()
}

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

    let mut module_ids_artifact = compilation.module_ids_artifact.steal();

    // Call beforeModuleIds hook - allows plugins to assign custom IDs
    let modules_needing_ids = get_modules_needing_ids(compilation, &module_ids_artifact);
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .before_module_ids
      .call(compilation, &modules_needing_ids, &mut module_ids_artifact)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.beforeModuleIds"))?;

    // Put artifact back so moduleIds plugins can see custom IDs from beforeModuleIds
    // when they call get_used_module_ids_and_modules
    compilation.module_ids_artifact = module_ids_artifact.into();

    let mut diagnostics = vec![];
    let mut module_ids_artifact = compilation.module_ids_artifact.steal();
    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .module_ids
      .call(compilation, &mut module_ids_artifact, &mut diagnostics)
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;
    compilation.module_ids_artifact = module_ids_artifact.into();
    compilation.extend_diagnostics(diagnostics);
    Ok(())
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_module_ids(compilation).await;
  }
}
