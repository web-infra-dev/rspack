use std::path::PathBuf;

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_paths::Utf8PathBuf;
use rspack_regex::RspackRegex;

// A compiler holds one value, so boxing the persistent options would save nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum CacheOptions {
  Disabled,
  Memory {
    /// The maximum number of generations to keep in memory.
    ///
    /// For example, if `max_generations` is set to 1,
    /// the cache will be removed if it's not accessed for 1 compilation generation.
    max_generations: u32,
  },
  Persistent(PersistentCacheOptions),
}

pub type BuildDepsOptions = Vec<PathBuf>;

/// Storage options shared by the cache backends.
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum StorageOptions {
  FileSystem {
    #[cacheable(with=As<PortablePath>)]
    directory: Utf8PathBuf,
    #[cacheable(with=As<PortablePath>)]
    location: Utf8PathBuf,
  },
}

/// The root that portable cache entries are rebased against, in both of its spellings.
///
/// It is cloned with [`PersistentCacheOptions`], which copies both paths.
#[derive(Debug, Clone)]
pub struct PortableRoot {
  /// The root as `context` spells it. Paths under it are stored relative to it and restored under
  /// the reader's spelling.
  pub root: Utf8PathBuf,
  /// The realpath of `root`. The resolver realpaths resources, so paths under this spelling are
  /// rebased as well.
  pub canonical_root: Utf8PathBuf,
}

/// Options shared by the cache backends.
#[derive(Debug, Clone)]
pub struct PersistentCacheOptions {
  pub build_dependencies: BuildDepsOptions,
  pub version: String,
  pub storage: StorageOptions,
  pub seed_donors: Vec<Utf8PathBuf>,
  pub portable_root: Option<PortableRoot>,
  /// Whether absolute requests that loaders generate are spelled relative to their module.
  pub rebase_loader_requests: bool,
  /// Whether to warn about modules that `rebase_loader_requests` would rewrite.
  pub warn_loader_requests: bool,
  /// The resource patterns of every applied `IgnorePlugin`. A loader request that one of them
  /// tells apart from its relative spelling is kept as written.
  pub ignored_requests: Vec<RspackRegex>,
  pub readonly: bool,
  /// Filesystem cache max age in seconds.
  pub max_age: u64,
  /// Number of generations to retain entries in the memory front cache.
  pub max_memory_generations: MaxMemoryGenerations,
}

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
