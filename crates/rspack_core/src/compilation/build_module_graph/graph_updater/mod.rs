mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::Error;
use rspack_paths::InternedPathSet;
use rustc_hash::FxHashSet;

use self::{cutout::Cutout, repair::repair};
use super::BuildModuleGraphArtifact;
use crate::{Compilation, DependencyId, ExportsInfoArtifact};

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

/// An update failure together with the artifacts that were being updated.
///
/// Graph updates consume their artifacts so they can move them into the repair
/// task context. Keeping the artifacts on an error lets callers restore their
/// phase state instead of leaving a `StealCell` empty.
#[derive(Debug)]
pub struct UpdateModuleGraphError {
  pub error: Error,
  pub artifact: BuildModuleGraphArtifact,
  pub exports_info_artifact: ExportsInfoArtifact,
}

impl From<UpdateModuleGraphError> for Error {
  fn from(error: UpdateModuleGraphError) -> Self {
    error.error
  }
}

/// Update module graph through `UpdateParam`
pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  mut exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> std::result::Result<(BuildModuleGraphArtifact, ExportsInfoArtifact), UpdateModuleGraphError> {
  let mut cutout = Cutout::default();

  let build_dependencies = cutout.cutout_artifact(compilation, &mut artifact, params);

  let revoked_modules = artifact.revoked_modules().copied().collect();
  if let Err(error) = compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(compilation, &revoked_modules)
    .await
  {
    cutout.fix_artifact(&mut artifact);
    return Err(UpdateModuleGraphError {
      error,
      artifact,
      exports_info_artifact,
    });
  }

  (artifact, exports_info_artifact) = match repair(
    compilation,
    artifact,
    exports_info_artifact,
    build_dependencies,
  )
  .await
  {
    Ok(result) => result,
    Err(mut error) => {
      cutout.fix_artifact(&mut error.artifact);
      return Err(error);
    }
  };
  cutout.fix_artifact(&mut artifact);
  let module_graph = artifact.get_module_graph();
  for id in artifact.affected_modules.active() {
    if let Some(module) = module_graph.module_by_identifier(id) {
      module.freeze_build_meta();
    }
  }
  Ok((artifact, exports_info_artifact))
}
