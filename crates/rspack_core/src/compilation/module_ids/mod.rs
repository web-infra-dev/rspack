use super::*;
use crate::logger::Logger;

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

  if !compilation
    .incremental
    .passes_enabled(IncrementalPasses::MODULE_IDS)
  {
    compilation.module_ids_artifact.clear();
  }

  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);

  let modules_needing_ids = get_modules_needing_ids(compilation, &module_ids_artifact);
  plugin_driver
    .compilation_hooks
    .before_module_ids
    .call(compilation, &modules_needing_ids, &mut module_ids_artifact)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.beforeModuleIds"))?;

  compilation.module_ids_artifact = module_ids_artifact;

  let mut diagnostics = vec![];
  let mut module_ids_artifact = mem::take(&mut compilation.module_ids_artifact);
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
