mod build_deps;

use rspack_cacheable::cacheable;
use rspack_paths::{InternedPath, InternedPathSet};

pub use self::build_deps::{BuildDeps, BuildDepsValidationResult};
use crate::{FileSystemInfo, Snapshot, SnapshotStrategyOptions};

#[cacheable]
#[derive(Debug, Default)]
pub(super) struct BuildDependenciesSnapshot {
  dependencies: InternedPathSet,
  snapshot: Snapshot,
}

impl BuildDependenciesSnapshot {
  pub(super) async fn validate(
    &self,
    file_system_info: &FileSystemInfo,
    build_deps: &BuildDeps,
  ) -> BuildDepsValidationResult {
    build_deps
      .validate_snapshot(file_system_info, &self.snapshot, self.dependencies.len())
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
    let snapshot = file_system_info
      .create_snapshot(
        None,
        std::iter::empty(),
        added.iter().cloned(),
        std::iter::empty(),
        SnapshotStrategyOptions::hash(),
      )
      .await;
    self.dependencies.extend(added);
    self.snapshot = file_system_info.merge_snapshots(std::mem::take(&mut self.snapshot), snapshot);
  }
}
