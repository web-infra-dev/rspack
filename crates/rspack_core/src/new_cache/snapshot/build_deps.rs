use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathSet};

use super::{FileSystemInfo, Snapshot, SnapshotValidationResult};
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

#[derive(Debug, Default)]
pub struct ResolvedBuildDependencies {
  pub(crate) dependencies: InternedPathSet,
  pub(crate) files: InternedPathSet,
  pub(crate) contexts: InternedPathSet,
  pub(crate) missing: InternedPathSet,
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
  ) -> ResolvedBuildDependencies {
    let mut helper = Helper::new(self.fs.clone(), self.logger.clone());
    let mut resolved = ResolvedBuildDependencies::default();
    let mut queue = VecDeque::new();
    queue.extend(self.pending.iter().cloned());
    queue.extend(paths);

    while let Some(dependency) = queue.pop_front() {
      if current.contains(&dependency) || !resolved.dependencies.insert(dependency.clone()) {
        continue;
      }
      match self.fs.metadata(dependency.assert_utf8()).await {
        Ok(metadata) if metadata.is_directory => {
          resolved.contexts.insert(dependency.clone());
        }
        Ok(_) => {
          resolved.files.insert(dependency.clone());
        }
        Err(_) => {
          resolved.missing.insert(dependency.clone());
        }
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
    resolved
  }

  /// Validate build dependencies.
  ///
  /// If any build dependency changed, this method returns an invalid result.
  pub async fn validate_snapshot(
    &self,
    file_system_info: &FileSystemInfo,
    snapshot: &Snapshot,
    dependencies: &InternedPathSet,
    tracked_files: usize,
  ) -> Result<BuildDepsValidationResult> {
    let validation = file_system_info.check_snapshot_valid(snapshot).await?;
    let pending = self
      .pending
      .iter()
      .filter(|path| !dependencies.contains(*path))
      .cloned()
      .collect::<InternedPathSet>();
    match validation {
      SnapshotValidationResult::Valid if pending.is_empty() => {
        Ok(BuildDepsValidationResult::Valid { tracked_files })
      }
      SnapshotValidationResult::Valid => Ok(BuildDepsValidationResult::Invalid {
        modified_files: pending,
        removed_files: Default::default(),
      }),
      SnapshotValidationResult::Invalid {
        mut modified_files,
        removed_files,
      } => {
        modified_files.extend(pending);
        Ok(BuildDepsValidationResult::Invalid {
          modified_files,
          removed_files,
        })
      }
    }
  }
}
