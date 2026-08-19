use std::{collections::VecDeque, path::PathBuf, sync::Arc};

use rspack_error::Result;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{ArcPath, ArcPathSet, AssertUtf8};

use super::{BuildDependenciesSnapshot, CacheMeta, Snapshot};
use crate::{
  CompilationLogger,
  cache::persistent::{
    build_dependencies::{Helper, is_node_package_path},
    codec::CacheCodec,
  },
};

pub type BuildDepsOptions = Vec<PathBuf>;

#[derive(Debug)]
pub enum BuildDepsValidationResult {
  InvalidVersion,
  Valid {
    tracked_files: usize,
  },
  Invalid {
    modified_files: ArcPathSet,
    removed_files: ArcPathSet,
  },
}

/// Build dependencies manager.
#[derive(Debug)]
pub struct BuildDeps {
  /// Dependencies configured at startup and added on the next store.
  pending: ArcPathSet,
  data: BuildDependenciesSnapshot,
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
        .map(|path| ArcPath::from(path.as_path()))
        .collect(),
      data: Default::default(),
      fs,
      logger,
    }
  }

  /// Update build dependencies and serialize the complete snapshot.
  ///
  /// For performance reasons, recursive searches stop at dependencies in
  /// `node_modules`.
  pub async fn create_snapshot(
    &mut self,
    codec: &CacheCodec,
    snapshot: &Snapshot,
    data: impl Iterator<Item = ArcPath>,
    rspack_pkg_version: &str,
    cache_version: &str,
  ) -> Result<Vec<u8>> {
    let mut helper = Helper::new(self.fs.clone(), self.logger.clone());
    let mut added = ArcPathSet::default();
    let mut queue = VecDeque::new();
    queue.extend(self.pending.iter().cloned());
    queue.extend(data);

    while let Some(current) = queue.pop_front() {
      if self.data.dependencies.contains(&current) || !added.insert(current.clone()) {
        continue;
      }
      if is_node_package_path(&current) {
        continue;
      }
      if let Some(children) = helper.resolve(current.assert_utf8()).await {
        queue.extend(
          children
            .into_iter()
            .map(|path| ArcPath::from(path.as_path())),
        );
      }
    }

    let snapshots = snapshot.add(added.iter().cloned()).await;
    self.data.dependencies.extend(added);
    self.data.snapshots.extend(snapshots);
    self.pending.clear();
    let meta = CacheMeta {
      rspack_pkg_version: rspack_pkg_version.to_string(),
      cache_version: cache_version.to_string(),
      build_dependencies: std::mem::take(&mut self.data),
    };
    let result = codec.encode(&meta);
    self.data = meta.build_dependencies;
    result
  }

  /// Validate build dependencies.
  ///
  /// If any build dependency changed, this method returns an invalid result.
  pub async fn validate_snapshot(
    &mut self,
    codec: &CacheCodec,
    snapshot: &Snapshot,
    data: Option<&[u8]>,
    rspack_pkg_version: &str,
    cache_version: &str,
  ) -> Result<BuildDepsValidationResult> {
    let Some(data) = data else {
      return Ok(BuildDepsValidationResult::InvalidVersion);
    };
    let meta = codec.decode::<CacheMeta>(data)?;
    if meta.rspack_pkg_version != rspack_pkg_version || meta.cache_version != cache_version {
      return Ok(BuildDepsValidationResult::InvalidVersion);
    }
    let (modified_files, removed_files) = snapshot
      .calc_modified_paths(&meta.build_dependencies.snapshots)
      .await;
    if !modified_files.is_empty() || !removed_files.is_empty() {
      return Ok(BuildDepsValidationResult::Invalid {
        modified_files,
        removed_files,
      });
    }
    let tracked_files = meta.build_dependencies.dependencies.len();
    self.data = meta.build_dependencies;
    Ok(BuildDepsValidationResult::Valid { tracked_files })
  }
}
