use std::sync::Arc;

use futures::future::join3;
use rspack_fs::ReadableFileSystem;
use rspack_parallel::FutureConsumer;
use rspack_paths::{ArcPath, ArcPathSet};

use super::{Snapshot, SnapshotEntry};
use crate::cache::persistent::snapshot::{
  SnapshotOptions, SnapshotScope, StrategyHelper, ValidateResult,
};

/// Creates and validates filesystem snapshots stored by the new cache.
#[derive(Debug)]
pub struct FileSystemInfo {
  options: Arc<SnapshotOptions>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl FileSystemInfo {
  pub fn new(options: SnapshotOptions, fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      options: Arc::new(options),
      fs,
    }
  }

  async fn create_entries(
    helper: Arc<StrategyHelper>,
    paths: impl Iterator<Item = ArcPath>,
    scope: SnapshotScope,
  ) -> Vec<SnapshotEntry> {
    let mut entries = Vec::with_capacity(paths.size_hint().0);
    paths
      .map(|path| {
        let helper = helper.clone();
        async move {
          helper
            .create_strategy(&path, scope)
            .await
            .map(|strategy| SnapshotEntry { path, strategy })
        }
      })
      .fut_consume(|entry| entries.extend(entry))
      .await;
    entries
  }

  #[tracing::instrument("Cache::FileSystemInfo::create_build_dependencies_snapshot", skip_all)]
  pub async fn create_build_dependencies_snapshot(
    &self,
    paths: impl Iterator<Item = ArcPath>,
  ) -> Vec<SnapshotEntry> {
    Self::create_entries(
      Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone())),
      paths,
      SnapshotScope::BUILD,
    )
    .await
  }

  /// Creates a snapshot from file, context, and missing dependencies.
  ///
  /// This follows webpack's `FileSystemInfo.createSnapshot` structure:
  /// <https://github.com/webpack/webpack/blob/main/lib/FileSystemInfo.js#L2534-L3017>
  #[tracing::instrument("Cache::FileSystemInfo::create_snapshot", skip_all)]
  pub async fn create_snapshot(
    &self,
    file_dependencies: impl Iterator<Item = ArcPath>,
    context_dependencies: impl Iterator<Item = ArcPath>,
    missing_dependencies: impl Iterator<Item = ArcPath>,
  ) -> Snapshot {
    let helper = Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone()));
    let (file_dependencies, context_dependencies, missing_dependencies) = join3(
      Self::create_entries(helper.clone(), file_dependencies, SnapshotScope::FILE),
      Self::create_entries(helper.clone(), context_dependencies, SnapshotScope::CONTEXT),
      Self::create_entries(helper, missing_dependencies, SnapshotScope::MISSING),
    )
    .await;
    Snapshot {
      file_dependencies,
      context_dependencies,
      missing_dependencies,
    }
  }

  #[tracing::instrument("Cache::FileSystemInfo::check_snapshot_valid", skip_all)]
  pub async fn check_snapshot_valid(&self, snapshot: &Snapshot) -> bool {
    let helper = Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone()));
    for entry in snapshot
      .file_dependencies
      .iter()
      .chain(&snapshot.context_dependencies)
      .chain(&snapshot.missing_dependencies)
    {
      if !matches!(
        helper.validate(&entry.path, &entry.strategy).await,
        ValidateResult::NoChanged
      ) {
        return false;
      }
    }
    true
  }

  #[tracing::instrument("Cache::FileSystemInfo::calc_modified_paths", skip_all)]
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
