mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use self::{cutout::Cutout, repair::repair};
use super::{MakeArtifact, MakeArtifactState};
use crate::{Compilation, DependencyId};

/// The param to update module graph
#[derive(Debug, Clone)]
pub enum UpdateParam {
  /// Build some entries, this param will only ensure that those entries are built,
  /// but will not remove entries that are not in this lists.
  BuildEntry(HashSet<DependencyId>),
  /// Build some entries and clean up the entries that not in this list.
  BuildEntryAndClean(HashSet<DependencyId>),
  /// Build the module which module.need_build is true, i.e. modules where loader.cacheable is false
  CheckNeedBuild,
  /// Build the module and dependency which depend on these modified file.
  ModifiedFiles(HashSet<ArcPath>),
  /// Build the module and dependency which depend on these removed file.
  RemovedFiles(HashSet<ArcPath>),
  /// Force build some dependencies.
  ForceBuildDeps(HashSet<DependencyId>),
  /// Force build some modules.
  ForceBuildModules(IdentifierSet),
  /// Need check isolated modules.
  CheckIsolatedModules(IdentifierSet),
}

/// Update module graph through `UpdateParam`
pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<UpdateParam>,
) -> Result<MakeArtifact> {
  artifact.state = MakeArtifactState::Initialized;
  let mut cutout = Cutout::default();

  let build_dependencies = cutout.cutout_artifact(compilation, &mut artifact, params);

  compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(compilation, &artifact.revoked_modules)
    .await?;

  artifact = repair(compilation, artifact, build_dependencies).await?;
  cutout.fix_artifact(&mut artifact);
  Ok(artifact)
}
