use std::path::PathBuf;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_paths::Utf8PathBuf;

use super::SnapshotOptions;

pub type BuildDepsOptions = Vec<PathBuf>;

/// Storage options shared by the cache backends.
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum StorageOptions {
  FileSystem {
    #[cacheable(with=As<PortablePath>)]
    directory: Utf8PathBuf,
  },
}

/// Persistent cache options shared by the cache backends.
#[derive(Debug, Clone)]
pub struct PersistentCacheOptions {
  pub build_dependencies: BuildDepsOptions,
  pub version: String,
  pub snapshot: SnapshotOptions,
  pub storage: StorageOptions,
  pub portable: bool,
  pub readonly: bool,
  /// Filesystem cache max age in seconds.
  pub max_age: u64,
}
