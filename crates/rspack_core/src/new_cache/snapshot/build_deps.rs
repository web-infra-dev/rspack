use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathSet};

use super::{Snapshot, SnapshotEntry};
use crate::{
  CompilationLogger,
  cache::persistent::build_dependencies::{Helper, is_node_package_path},
};

pub type BuildDepsOptions = Vec<PathBuf>;

#[derive(Debug)]
pub enum BuildDepsValidationResult {
  Valid {
    tracked_files: usize,
  },
  Invalid {
    modified_files: InternedPathSet,
    removed_files: InternedPathSet,
  },
}

/// Build dependencies manager.
#[derive(Debug)]
pub struct BuildDeps {
  /// Dependencies configured at startup and added on the next store.
  pending: InternedPathSet,
  fs: Arc<dyn ReadableFileSystem>,
  logger: CompilationLogger,
}

impl BuildDeps {
  pub fn new(
    options: &BuildDepsOptions,
    fs: Arc<dyn ReadableFileSystem>,
    logger: CompilationLogger,
  ) -> Self {
    Self {
      pending: options
        .iter()
        .map(|path| InternedPath::from(path.as_path()))
        .collect(),
      fs,
      logger,
    }
  }

  /// Resolve build dependencies that are not in the current snapshot.
  ///
  /// For performance reasons, recursive searches stop at dependencies in
  /// `node_modules`.
  pub async fn resolve_dependencies(
    &mut self,
    current: &InternedPathSet,
    paths: impl Iterator<Item = InternedPath>,
  ) -> InternedPathSet {
    let mut helper = Helper::new(self.fs.clone(), self.logger.clone());
    let mut added = InternedPathSet::default();
    let mut queue = VecDeque::new();
    queue.extend(self.pending.iter().cloned());
    queue.extend(paths);

    while let Some(dependency) = queue.pop_front() {
      if current.contains(&dependency) || !added.insert(dependency.clone()) {
        continue;
      }
      if is_node_package_path(&dependency) {
        continue;
      }
      if let Some(children) = helper.resolve(dependency.assert_utf8()).await {
        queue.extend(
          children
            .into_iter()
            .map(|path| InternedPath::from(path.as_path())),
        );
      }
    }

    self.pending.clear();
    added
  }

  /// Validate build dependencies.
  ///
  /// If any build dependency changed, this method returns an invalid result.
  pub async fn validate_snapshot(
    &self,
    snapshot: &Snapshot,
    entries: &[SnapshotEntry],
    tracked_files: usize,
  ) -> BuildDepsValidationResult {
    let (modified_files, removed_files) = snapshot.calc_modified_paths(entries).await;
    if !modified_files.is_empty() || !removed_files.is_empty() {
      return BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      };
    }
    BuildDepsValidationResult::Valid { tracked_files }
  }
}
