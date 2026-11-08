use std::{collections::VecDeque, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{AssertUtf8, InternedPath, InternedPathSet};
use rustc_hash::FxHashSet as HashSet;

use super::{
  snapshot::{Snapshot, SnapshotScope},
  storage::Storage,
};
use crate::{
  BuildDepsOptions, CompilationLogger,
  cache::{BuildDependencyHelper, is_node_package_path},
};

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

/// Build dependencies manager
#[derive(Debug)]
pub struct BuildDeps {
  /// The build dependencies has been added to snapshot.
  ///
  /// This field is used to avoid adding duplicate build dependencies to the snapshot.
  added: InternedPathSet,
  /// The pending dependencies.
  ///
  /// The next time the add method is called, this path will be additionally added.
  pending: InternedPathSet,
  /// The snapshot which is used to save build dependencies.
  snapshot: Arc<Snapshot>,
  fs: Arc<dyn ReadableFileSystem>,
}

impl BuildDeps {
  pub fn new(
    options: &BuildDepsOptions,
    fs: Arc<dyn ReadableFileSystem>,
    snapshot: Arc<Snapshot>,
  ) -> Self {
    Self {
      added: Default::default(),
      pending: options
        .iter()
        .map(|v| InternedPath::from(v.as_path()))
        .collect(),
      snapshot,
      fs,
    }
  }

  /// Add build dependencies
  ///
  /// For performance reasons, recursive searches will stop for build dependencies in node_modules.
  pub async fn add(
    &mut self,
    storage: &mut dyn Storage,
    data: impl Iterator<Item = InternedPath>,
    logger: CompilationLogger,
  ) {
    let mut helper = BuildDependencyHelper::new(self.fs.clone(), logger);
    let mut new_deps = HashSet::default();
    let mut queue = VecDeque::new();
    queue.extend(std::mem::take(&mut self.pending));
    queue.extend(data);
    while let Some(current) = queue.pop_front() {
      if !self.added.insert(current.clone()) {
        continue;
      }
      new_deps.insert(current.clone());
      if is_node_package_path(&current) {
        // node package path skip recursive search.
        continue;
      }
      if let Some(children) = helper.resolve(current.assert_utf8()).await {
        queue.extend(children.iter().map(|item| item.as_path().into()));
      }
    }

    self
      .snapshot
      .add(storage, SnapshotScope::BUILD, new_deps.into_iter())
      .await;
  }

  /// Validate build dependencies
  ///
  /// If any build dependencies have changed, this method will return an invalid result.
  pub async fn validate(
    &mut self,
    storage: &dyn Storage,
    has_previous_cache: bool,
  ) -> Result<BuildDepsValidationResult> {
    let (_, mut modified_files, removed_files, no_changed_files) = self
      .snapshot
      .calc_modified_paths(storage, SnapshotScope::BUILD)
      .await?;

    if has_previous_cache {
      modified_files.extend(
        self
          .pending
          .iter()
          .filter(|path| !no_changed_files.contains(*path))
          .cloned(),
      );
    }

    if !modified_files.is_empty() || !removed_files.is_empty() {
      return Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      });
    }
    let tracked_files = no_changed_files.len();
    self.added = no_changed_files;
    Ok(BuildDepsValidationResult::Valid { tracked_files })
  }
}
