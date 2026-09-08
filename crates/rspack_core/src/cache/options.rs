use std::path::PathBuf;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_paths::Utf8PathBuf;

use super::SnapshotOptions;
use crate::{CompilerOptions, RuntimeTemplateRenderMode};

pub type BuildDepsOptions = Vec<PathBuf>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
  /// Number of generations to retain entries in the memory front cache.
  pub max_memory_generations: MaxMemoryGenerations,
}

impl PersistentCacheOptions {
  pub(crate) fn version(&self, compiler_options: &CompilerOptions) -> String {
    // Parsing can embed runtime identifiers in dependencies, so invalidate
    // persistent entries before restoring modules when their render mode changes.
    format!(
      "{}|{:?}",
      self.version,
      RuntimeTemplateRenderMode::from_options(compiler_options)
    )
  }
}
