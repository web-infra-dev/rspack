mod build_deps;

use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathSet};

pub use self::build_deps::{BuildDeps, BuildDepsValidationResult};
use crate::cache::persistent::snapshot::{
  SnapshotOptions, SnapshotStrategyOptions, Strategy, StrategyHelper, ValidateResult,
};

#[cacheable]
#[derive(Debug)]
pub struct SnapshotEntry {
  path: ArcPath,
  strategy: Strategy,
}

#[cacheable]
#[derive(Debug, Default)]
pub(super) struct BuildDependenciesSnapshot {
  dependencies: ArcPathSet,
  snapshots: Vec<SnapshotEntry>,
}

impl BuildDependenciesSnapshot {
  pub(super) async fn validate(
    &self,
    snapshot: &Snapshot,
    build_deps: &BuildDeps,
  ) -> BuildDepsValidationResult {
    build_deps
      .validate_snapshot(snapshot, &self.snapshots, self.dependencies.len())
      .await
  }

  pub(super) async fn update(
    &mut self,
    snapshot: &Snapshot,
    build_deps: &mut BuildDeps,
    paths: impl Iterator<Item = ArcPath>,
  ) {
    let added = build_deps
      .resolve_dependencies(&self.dependencies, paths)
      .await;
    let snapshots = snapshot.add(added.iter().cloned()).await;
    self.dependencies.extend(added);
    self.snapshots.extend(snapshots);
  }
}

/// Creates and validates filesystem snapshots stored by the new cache.
#[derive(Debug)]
pub struct Snapshot {
  options: Arc<SnapshotOptions>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl Snapshot {
  pub fn new(options: SnapshotOptions, fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      options: Arc::new(options),
      fs,
    }
  }

  async fn calc_strategy(&self, helper: &StrategyHelper, path: &ArcPath) -> Option<Strategy> {
    let path_str = path.to_string_lossy();
    if self.options.is_immutable_path(&path_str) {
      return None;
    }
    if self.options.is_managed_path(&path_str)
      && let Some(strategy) = helper.package_version(path).await
    {
      return Some(strategy);
    }
    Some(
      helper
        .dir_strategy(path, SnapshotStrategyOptions::hash())
        .await,
    )
  }

  #[tracing::instrument("Cache::Snapshot::add", skip_all)]
  pub async fn add(&self, paths: impl Iterator<Item = ArcPath>) -> Vec<SnapshotEntry> {
    let helper = StrategyHelper::new(self.fs.clone(), self.options.clone());
    let mut entries = Vec::with_capacity(paths.size_hint().0);
    for path in paths {
      if let Some(strategy) = self.calc_strategy(&helper, &path).await {
        entries.push(SnapshotEntry { path, strategy });
      }
    }
    entries
  }

  #[tracing::instrument("Cache::Snapshot::calc_modified_paths", skip_all)]
  pub async fn calc_modified_paths(&self, entries: &[SnapshotEntry]) -> (ArcPathSet, ArcPathSet) {
    let helper = StrategyHelper::new(self.fs.clone(), self.options.clone());
    let mut modified_files = ArcPathSet::default();
    let mut removed_files = ArcPathSet::default();
    for entry in entries {
      match helper.validate(&entry.path, &entry.strategy).await {
        ValidateResult::Modified => {
          modified_files.insert(entry.path.clone());
        }
        ValidateResult::Deleted => {
          removed_files.insert(entry.path.clone());
        }
        ValidateResult::NoChanged => {}
      }
    }
    (modified_files, removed_files)
  }
}
