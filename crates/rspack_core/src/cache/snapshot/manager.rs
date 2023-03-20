use std::{
  hash::BuildHasherDefault,
  path::{Path, PathBuf},
  time::SystemTime,
};

use dashmap::{DashMap, DashSet};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHasher};

use super::Snapshot;
use crate::{calc_hash, SnapshotOptions, SnapshotStrategy};

/// SnapshotManager is a tools to create or check snapshot
///
/// this struct has cache to improve create and check speed.
/// we need to keep the same lifetime as compilation, or clear cache after compilation is done.
#[derive(Debug)]
pub struct SnapshotManager {
  /// global snapshot options
  options: SnapshotOptions,
  /// cache file update time
  update_time_cache: DashMap<PathBuf, SystemTime, BuildHasherDefault<FxHasher>>,
  /// cache file hash
  hash_cache: DashMap<PathBuf, u64, BuildHasherDefault<FxHasher>>,
  /// has modified file
  modified_files: DashSet<PathBuf>,
}

impl SnapshotManager {
  pub fn new(options: SnapshotOptions) -> Self {
    Self {
      options,
      update_time_cache: Default::default(),
      hash_cache: Default::default(),
      modified_files: Default::default(),
    }
  }

  pub async fn create_snapshot<F>(&self, paths: &[&Path], f: F) -> Result<Snapshot>
  where
    F: FnOnce(&SnapshotOptions) -> &SnapshotStrategy,
  {
    // TODO file_paths deduplication && calc immutable path
    let strategy = f(&self.options);
    let mut file_update_times = HashMap::default();
    file_update_times.reserve(paths.len());
    let mut file_hashes = HashMap::default();
    file_hashes.reserve(paths.len());
    if strategy.timestamp {
      for &path in paths {
        file_update_times.insert(path.to_owned(), SystemTime::now());
      }
    }
    if strategy.hash {
      let hash_cache = &self.hash_cache;
      for &path in paths {
        let hash = match hash_cache.get(path) {
          Some(hash) => *hash,
          None => {
            let res = calc_hash(&tokio::fs::read(path).await?);
            hash_cache.insert(path.to_owned(), res);
            res
          }
        };
        file_hashes.insert(path.to_owned(), hash);
      }
    }

    Ok(Snapshot {
      file_update_times,
      file_hashes,
    })
  }

  pub async fn check_snapshot_valid(&self, snapshot: &Snapshot) -> Result<bool> {
    let Snapshot {
      file_update_times,
      file_hashes,
      ..
    } = snapshot;
    if !file_update_times.is_empty() {
      // check update time
      let update_time_cache = &self.update_time_cache;
      for (path, snapshot_time) in file_update_times {
        if self.modified_files.contains(path) {
          return Ok(false);
        }

        let update_time = match update_time_cache.get(path) {
          Some(t) => *t,
          None => {
            let t = tokio::fs::metadata(path).await?.modified()?;
            update_time_cache.insert(path.clone(), t);
            t
          }
        };

        if snapshot_time < &update_time {
          return Ok(false);
        }
      }
    }

    if !file_hashes.is_empty() {
      // check file hash
      let hash_cache = &self.hash_cache;
      for (path, snapshot_hash) in file_hashes {
        if self.modified_files.contains(path) {
          return Ok(false);
        }

        let current_hash = match hash_cache.get(path) {
          Some(h) => *h,
          None => {
            let res = calc_hash(&tokio::fs::read(path).await?);
            hash_cache.insert(path.clone(), res);
            res
          }
        };
        if snapshot_hash != &current_hash {
          return Ok(false);
        }
      }
    }

    Ok(true)
  }

  pub fn clear(&self) {
    self.update_time_cache.clear();
    self.hash_cache.clear();
    self.modified_files.clear();
  }

  pub fn set_modified_files(&self, files: Vec<PathBuf>) {
    for item in files {
      self.modified_files.insert(item);
    }
  }
}
