mod build_deps;
mod file_system_info;

use rspack_cacheable::cacheable;
use rspack_paths::{InternedPath, InternedPathSet};

pub use self::{
  build_deps::{BuildDeps, BuildDepsValidationResult},
  file_system_info::FileSystemInfo,
};
use crate::cache::persistent::snapshot::Strategy;

#[cacheable]
#[derive(Debug, Clone)]
pub struct SnapshotEntry {
  path: InternedPath,
  strategy: Strategy,
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct Snapshot {
  file_dependencies: Vec<SnapshotEntry>,
  context_dependencies: Vec<SnapshotEntry>,
  missing_dependencies: Vec<SnapshotEntry>,
}

#[cacheable]
#[derive(Debug, Default)]
pub(super) struct BuildDependenciesSnapshot {
  dependencies: InternedPathSet,
  snapshots: Vec<SnapshotEntry>,
}

impl BuildDependenciesSnapshot {
  pub(super) async fn validate(
    &self,
    file_system_info: &FileSystemInfo,
    build_deps: &BuildDeps,
  ) -> BuildDepsValidationResult {
    build_deps
      .validate_snapshot(file_system_info, &self.snapshots, self.dependencies.len())
      .await
  }

  pub(super) async fn update(
    &mut self,
    file_system_info: &FileSystemInfo,
    build_deps: &mut BuildDeps,
    paths: impl Iterator<Item = InternedPath>,
  ) {
    let added = build_deps
      .resolve_dependencies(&self.dependencies, paths)
      .await;
    let snapshots = file_system_info
      .create_build_dependencies_snapshot(added.iter().cloned())
      .await;
    self.dependencies.extend(added);
    self.snapshots.extend(snapshots);
  }
}
