use async_trait::async_trait;

use super::*;
use crate::compilation::pass::PassExt;

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
        && (chunk_graph.get_number_of_module_chunks(m.identifier()) != 0
          || m.build_meta().is_css_module()
          || m.build_meta().need_id_in_concatenation())
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

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::MODULE_IDS
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    // Check if MODULE_IDS pass is disabled, and clear artifact if needed
    if !compilation
      .incremental
      .passes_enabled(IncrementalPasses::MODULE_IDS)
    {
      compilation.module_ids_artifact.clear();
    }

    // Keep a copy of the current ids so the end of this pass can tell which
    // modules got a different id. If a copy already exists, the last rebuild
    // failed before module hashing: keep that older copy, because the module
    // hashes were built from it. Cold builds don't track changes, so they skip
    // the copy.
    if compilation.incremental.mutations_writable()
      && compilation.module_ids_diff_artifact.previous_ids.is_none()
    {
      compilation.module_ids_diff_artifact.previous_ids =
        Some((*compilation.module_ids_artifact).clone());
    }

    let module_ids_artifact = compilation.module_ids_artifact.steal();
    let mut preserved_module_ids_artifact = ModuleIdsArtifact::default();

    // Call reviveModules hook - allows plugins to restore IDs from records
    if !compilation
      .plugin_driver
      .compilation_hooks
      .revive_modules
      .is_empty()
    {
      let modules_needing_ids =
        get_modules_needing_ids(compilation, &preserved_module_ids_artifact);
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .revive_modules
        .call(
          compilation,
          &modules_needing_ids,
          &mut preserved_module_ids_artifact,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.reviveModules"))?;
    }

    // Call beforeModuleIds hook - allows plugins to assign custom IDs
    if !compilation
      .plugin_driver
      .compilation_hooks
      .before_module_ids
      .is_empty()
    {
      let modules_needing_ids =
        get_modules_needing_ids(compilation, &preserved_module_ids_artifact);
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .before_module_ids
        .call(
          compilation,
          &modules_needing_ids,
          &mut preserved_module_ids_artifact,
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.beforeModuleIds"))?;
    }

    // Put the recovered artifact back before preserved IDs are merged.
    compilation.module_ids_artifact = module_ids_artifact.into();

    let mut diagnostics = vec![];
    let mut module_ids_artifact = compilation.module_ids_artifact.steal();

    // Merge IDs assigned by reviveModules and beforeModuleIds before running module ID plugins,
    // so every plugin sees them as reserved IDs. Plugins that reset global IDs retain this
    // preserved subset.
    for (module, id) in preserved_module_ids_artifact.iter() {
      ChunkGraph::set_module_id(&mut module_ids_artifact, *module, id.clone());
    }

    compilation
      .plugin_driver
      .clone()
      .compilation_hooks
      .module_ids
      .call(
        compilation,
        &mut module_ids_artifact,
        &preserved_module_ids_artifact,
        &mut diagnostics,
      )
      .await
      .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.moduleIds"))?;

    if !compilation
      .plugin_driver
      .compilation_hooks
      .record_modules
      .is_empty()
    {
      compilation
        .plugin_driver
        .clone()
        .compilation_hooks
        .record_modules
        .call(compilation, &module_ids_artifact)
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.recordModules"))?;
    }
    // Record every module whose id changed, including modules that still exist
    // but no longer have an id. Compare only the final ids: plugins may clear
    // and reassign ids during this pass, and those in-between states don't
    // matter. This must run before CreateModuleHashesPass, which reads these
    // mutations once to decide which modules need new hashes (modules whose id
    // changed, and the modules that reference them).
    if let Some(mut mutations) = compilation.incremental.mutations_write() {
      for (module, id) in module_ids_artifact.iter() {
        if compilation
          .module_ids_diff_artifact
          .previous_ids
          .as_ref()
          .and_then(|ids| ids.get(module))
          != Some(id)
        {
          mutations.add(Mutation::ModuleSetId { module: *module });
        }
      }
      if let Some(previous_ids) = &compilation.module_ids_diff_artifact.previous_ids {
        let module_graph = compilation.get_module_graph();
        for module in previous_ids.keys() {
          if !module_ids_artifact.contains_key(module)
            && module_graph.module_by_identifier(module).is_some()
          {
            mutations.add(Mutation::ModuleSetId { module: *module });
          }
        }
      }
    }
    compilation.module_ids_artifact = module_ids_artifact.into();
    compilation.extend_diagnostics(diagnostics);
    Ok(())
  }
}
