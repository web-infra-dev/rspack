mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

use self::{cutout::Cutout, repair::repair};
use super::{MakeArtifact, MakeArtifactState};
use crate::{Compilation, DependencyId};

#[derive(Debug, Clone)]
pub enum UpdateParam {
  BuildEntry(HashSet<DependencyId>),
  BuildEntryAndClean(HashSet<DependencyId>),
  CheckNeedBuild,
  ModifiedFiles(HashSet<ArcPath>),
  RemovedFiles(HashSet<ArcPath>),
  ForceBuildDeps(HashSet<DependencyId>),
  ForceBuildModules(IdentifierSet),
}

pub async fn update_module_graph(
  compilation: &Compilation,
  mut artifact: MakeArtifact,
  params: Vec<UpdateParam>,
) -> Result<MakeArtifact> {
  artifact.state = MakeArtifactState::Initialized;
  let mut cutout = Cutout::default();

  let build_dependencies = cutout.cutout_artifact(&mut artifact, params);

  compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(&artifact.revoked_modules)
    .await?;

  artifact = repair(compilation, artifact, build_dependencies).await?;
  cutout.fix_artifact(&mut artifact);
  Ok(artifact)
}
