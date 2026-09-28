use std::{path::PathBuf, time::Duration};

use rspack_paths::Utf8PathBuf;
#[cfg(allocative)]
use rspack_util::allocative;

pub use crate::legacy_cache::{BuildDepsOptions, PersistentCacheOptions, StorageOptions};

#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum CacheOptions {
  Disabled,
  Memory {
    /// The maximum number of generations to keep in memory.
    ///
    /// For example, if `max_generations` is set to 1,
    /// the cache will be removed if it's not accessed for 1 compilation generation.
    max_generations: u32,
  },
  FileSystem(FileSystemCacheOptions),
  Persistent(PersistentCacheOptions),
}

/// Filesystem cache configuration, cloned with the compiler's cache options.
#[derive(Debug, Clone)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct FileSystemCacheOptions {
  pub build_dependencies: Vec<PathBuf>,
  pub cache_directory: Utf8PathBuf,
  pub cache_location: Utf8PathBuf,
  pub version: String,
  pub readonly: bool,
  pub max_memory_generations: MaxMemoryGenerations,
  pub idle_timeout: Duration,
  pub idle_timeout_for_initial_store: Duration,
  pub idle_timeout_after_large_changes: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub enum MaxMemoryGenerations {
  Disabled,
  Infinity,
  Finite(u32),
}

impl From<Option<u32>> for MaxMemoryGenerations {
  fn from(value: Option<u32>) -> Self {
    match value {
      Some(0) => Self::Disabled,
      Some(value) => Self::Finite(value),
      None => Self::Infinity,
    }
  }
}
