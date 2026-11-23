mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_paths::InternedPathSet;
use rustc_hash::FxHashSet;

use self::{cutout::Cutout, repair::repair};
use super::BuildModuleGraphArtifact;
use crate::{Compilation, DependencyId, ExportsInfoArtifact, ModuleGraph};

/// Expand `modules` across `is_module_executor_result()` dependencies.
///
/// Always propagates forward to generated modules so rebuilding a producer refreshes
/// its generated results. When `include_sibling_producers` is true (explicit
/// `rebuildModule`), also pulls in other producers of shared generated modules so
/// repair cannot recreate them from stale dependency data.
pub fn expand_module_executor_results(
  module_graph: &ModuleGraph,
  modules: &mut IdentifierSet,
  include_sibling_producers: bool,
) {
  let mut queue = modules.iter().copied().collect::<Vec<_>>();
  let mut expanded_generated = IdentifierSet::default();
  while let Some(module) = queue.pop() {
    for connection in module_graph.get_outgoing_connections(&module) {
      if !module_graph
        .dependency_by_id(&connection.dependency_id)
        .is_module_executor_result()
      {
        continue;
      }
      let generated = *connection.module_identifier();
      if module_graph.module_by_identifier(&generated).is_none()
        || !expanded_generated.insert(generated)
      {
        continue;
      }
      if modules.insert(generated) {
        queue.push(generated);
      }
      if !include_sibling_producers {
        continue;
      }
      for incoming in module_graph.get_incoming_connections(&generated) {
        if let Some(producer) = incoming.original_module_identifier
          && module_graph.module_by_identifier(&producer).is_some()
          && modules.insert(producer)
        {
          queue.push(producer);
        }
      }
    }
  }
}

/// The param to update module graph
#[derive(Debug, Clone)]
pub enum UpdateParam {
  /// Build some entries, this param will only ensure that those entries are built,
  /// but will not remove entries that are not in this lists.
  BuildEntry(FxHashSet<DependencyId>),
  /// Build some entries and clean up the entries that not in this list.
  BuildEntryAndClean(FxHashSet<DependencyId>),
  /// Build modules whose incremental rebuild check returns true, for example
  /// modules where `loader.cacheable` is false.
  CheckNeedBuild,
  /// Build the module and dependency which depend on these modified file.
  ModifiedFiles(InternedPathSet),
  /// Build the module and dependency which depend on these removed file.
  RemovedFiles(InternedPathSet),
  /// Force build some modules.
  ForceBuildModules(IdentifierSet),
}

/// Always returns ownership of the graph, including when repair fails.
/// A failed update must not be used by subsequent compilation passes.
pub struct ModuleGraphUpdateResult {
  pub artifact: BuildModuleGraphArtifact,
  pub exports_info_artifact: ExportsInfoArtifact,
  pub result: Result<()>,
}

/// Update module graph through `UpdateParam`
pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  mut exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> ModuleGraphUpdateResult {
  // Also covers rebuildModule and entry additions outside the make phase.
  let _loader_cache_guard = compilation
    .loader_cache()
    .upgrade()
    .map(|cache| cache.begin_build());
  let mut cutout = Cutout::default();

  let mut force_build_modules = IdentifierSet::default();
  let build_dependencies =
    cutout.cutout_artifact(compilation, &mut artifact, params, &mut force_build_modules);

  let revoked_modules = artifact.revoked_modules().copied().collect();
  let result = compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(compilation, &revoked_modules)
    .await;
  if let Err(error) = result {
    return ModuleGraphUpdateResult {
      artifact,
      exports_info_artifact,
      result: Err(error),
    };
  }

  let result;
  (artifact, exports_info_artifact, result) = repair(
    compilation,
    artifact,
    exports_info_artifact,
    build_dependencies,
    force_build_modules,
  )
  .await;
  if result.is_ok() {
    cutout.fix_artifact(&mut artifact);
    let module_graph = artifact.get_module_graph();
    for id in artifact.affected_modules.active() {
      if let Some(module) = module_graph.module_by_identifier(id) {
        module.freeze_build_meta();
      }
    }
  }
  ModuleGraphUpdateResult {
    artifact,
    exports_info_artifact,
    result,
  }
}
