mod build_deps;
mod strategy;

use std::sync::Arc;

use rspack_cacheable::cacheable;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{InternedPath, InternedPathSet};
use rspack_parallel::FutureConsumer;

pub use self::build_deps::{BuildDeps, BuildDepsValidationResult};
use self::strategy::{SnapshotScope, calc_strategy};
use crate::cache::persistent::snapshot::{
  SnapshotOptions, Strategy, StrategyHelper, ValidateResult,
};

#[cacheable]
#[derive(Debug)]
pub struct SnapshotEntry {
  path: InternedPath,
  strategy: Strategy,
}

#[cacheable]
#[derive(Debug, Default)]
pub struct ModuleSnapshot {
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
    paths: impl Iterator<Item = InternedPath>,
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

  async fn add_with_scopes(
    &self,
    paths: impl Iterator<Item = (SnapshotScope, InternedPath)>,
  ) -> Vec<(SnapshotScope, SnapshotEntry)> {
    let helper = Arc::new(StrategyHelper::new(self.fs.clone(), self.options.clone()));
    let options = self.options.clone();
    let mut entries = Vec::with_capacity(paths.size_hint().0);
    paths
      .map(|(scope, path)| {
        let helper = helper.clone();
        let options = options.clone();
        async move {
          calc_strategy(&options, &helper, &path, scope)
            .await
            .map(|strategy| (scope, SnapshotEntry { path, strategy }))
        }
      })
      .fut_consume(|entry| entries.extend(entry))
      .await;
    entries
  }

  #[tracing::instrument("Cache::Snapshot::add", skip_all)]
  pub async fn add(&self, paths: impl Iterator<Item = InternedPath>) -> Vec<SnapshotEntry> {
    self
      .add_with_scopes(paths.map(|path| (SnapshotScope::Build, path)))
      .await
      .into_iter()
      .map(|(_, entry)| entry)
      .collect()
  }

  #[tracing::instrument("Cache::Snapshot::create_module", skip_all)]
  pub async fn create_module(
    &self,
    file_dependencies: impl Iterator<Item = InternedPath>,
    context_dependencies: impl Iterator<Item = InternedPath>,
    missing_dependencies: impl Iterator<Item = InternedPath>,
  ) -> ModuleSnapshot {
    let entries = self
      .add_with_scopes(
        file_dependencies
          .map(|path| (SnapshotScope::File, path))
          .chain(context_dependencies.map(|path| (SnapshotScope::Context, path)))
          .chain(missing_dependencies.map(|path| (SnapshotScope::Missing, path))),
      )
      .await;
    let mut snapshot = ModuleSnapshot::default();
    for (scope, entry) in entries {
      match scope {
        SnapshotScope::File => snapshot.file_dependencies.push(entry),
        SnapshotScope::Context => snapshot.context_dependencies.push(entry),
        SnapshotScope::Missing => snapshot.missing_dependencies.push(entry),
        SnapshotScope::Build => unreachable!("module snapshots do not contain build dependencies"),
      }
    }
    snapshot
  }

  #[tracing::instrument("Cache::Snapshot::validate_module", skip_all)]
  pub async fn validate_module(&self, snapshot: &ModuleSnapshot) -> bool {
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

  #[tracing::instrument("Cache::Snapshot::calc_modified_paths", skip_all)]
  pub async fn calc_modified_paths(
    &self,
    entries: &[SnapshotEntry],
  ) -> (InternedPathSet, InternedPathSet) {
    let helper = StrategyHelper::new(self.fs.clone(), self.options.clone());
    let mut modified_files = InternedPathSet::default();
    let mut removed_files = InternedPathSet::default();
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
