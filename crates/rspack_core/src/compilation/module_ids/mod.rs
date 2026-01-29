use super::*;
use crate::logger::Logger;

/// Collects module identifiers that need ID assignment.
/// A module needs an ID if:
/// - It doesn't already have one assigned
/// - It needs an ID (need_id() returns true)
/// - It's part of at least one chunk
fn get_modules_needing_ids(
  compilation: &Compilation,
  module_ids_artifact: &ModuleIdsArtifact,
) -> IdentifierSet {
  let chunk_graph = &compilation.chunk_graph;
  compilation
    .get_module_graph()
    .modules()
    .values()
    .filter(|m| {
      m.need_id()
        && ChunkGraph::get_module_id(module_ids_artifact, m.identifier()).is_none()
        && chunk_graph.get_number_of_module_chunks(m.identifier()) != 0
    })
    .map(|m| m.identifier())
    .collect()
}

pub async fn module_ids_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("module ids");

  // Check if MODULE_IDS pass is disabled, and clear artifact if needed
  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULE_IDS)
  {
    compilation.module_ids_artifact.clear();
  }

  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);

  // Call beforeModuleIds hook - allows plugins to assign custom IDs
  let modules_needing_ids = get_modules_needing_ids(compilation, &module_ids_artifact);
  plugin_driver
    .compilation_hooks
    .before_module_ids
    .call(compilation, &modules_needing_ids, &mut module_ids_artifact)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.beforeModuleIds"))?;

  // Put artifact back so moduleIds plugins can see custom IDs from beforeModuleIds
  // when they call get_used_module_ids_and_modules
  compilation.module_ids_artifact = module_ids_artifact;

  // Call moduleIds hook - built-in ID assignment for remaining modules
  // Pass a reference to compilation's artifact directly so plugins can read AND write to it
  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);
  let mut diagnostics = vec![];
  plugin_driver
    .compilation_hooks
    .module_ids
    .call(compilation, &mut module_ids_artifact, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;

  compilation.module_ids_artifact = module_ids_artifact;
  compilation.extend_diagnostics(diagnostics);
  logger.time_end(start);
  Ok(())
}
