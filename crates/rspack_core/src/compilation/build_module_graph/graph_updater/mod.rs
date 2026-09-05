mod cutout;
pub mod repair;

use std::sync::Arc;

use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_paths::InternedPathSet;
use rustc_hash::FxHashSet;

use self::{cutout::Cutout, repair::repair};
use super::{BuildModuleGraphArtifact, MakeSession};
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

/// Update module graph through `UpdateParam`
pub async fn update_module_graph(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  update_module_graph_with_session(
    compilation,
    artifact,
    exports_info_artifact,
    params,
    compilation.make_session.clone(),
  )
  .await
}

pub(crate) async fn update_module_graph_with_session(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  mut exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
  session: Arc<MakeSession>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let mut cutout = Cutout::default();

  let build_dependencies = cutout.cutout_artifact(compilation, &mut artifact, params);

  let revoked_modules = artifact.revoked_modules().copied().collect();
  compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(compilation, &revoked_modules)
    .await?;

  (artifact, exports_info_artifact) = repair(
    compilation,
    artifact,
    exports_info_artifact,
    build_dependencies,
    session,
  )
  .await?;
  cutout.fix_artifact(&mut artifact);
  Ok((artifact, exports_info_artifact))
}
