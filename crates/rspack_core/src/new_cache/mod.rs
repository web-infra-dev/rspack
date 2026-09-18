mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod db;
mod etag;
mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;
pub(crate) mod snapshot;
mod validator;

use std::sync::Arc;

pub use cache::{Cache, CompilerCache};
pub use cache_facade::{CacheFacade, ItemCacheFacade, MultiItemCache};
pub use cache_key::CacheKey;
pub use cache_value::CacheValue;
pub use etag::Etag;
pub use file_cache_strategy::FileCacheStrategy;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
use rspack_fs::ReadableFileSystem;
pub use snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult};

use crate::{
  CompilerOptions, InfrastructureLogSink, InfrastructureLogger, MaxMemoryGenerations,
  cache::CacheCodec,
};

/// Creates cache storage independently of a compiler's namespace.
pub fn create_cache(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
) -> Cache {
  if !compiler_options.experiments.new_cache.is_enabled() {
    return Cache::new_disabled();
  }

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => {
      return Cache::new_disabled();
    }
    crate::CacheOptions::Persistent(_) => {
      unreachable!("persistent cache options must be converted before enabling newCache")
    }
    crate::CacheOptions::Memory {
      max_generations: _, /* TODO: old cache default to 1, change to 5 and pass to MemoryCache */
      ..
    } => {
      return Cache::new(Some(MemoryCache::new(5)), None);
    }
    crate::CacheOptions::FileSystem(options) => options,
  };

  let codec = Arc::new(CacheCodec::new(None));
  let logger = Arc::new(InfrastructureLogger::new(
    "rspack.cache.IdleFileCache",
    infrastructure_log_sink,
  ));
  let file_system_info = FileSystemInfo::new(
    input_filesystem,
    logger.get_child("rspack.FileSystemInfo"),
    compiler_options.snapshot.clone(),
    compiler_options.output.hash_function,
  );
  let base_path = options.cache_directory.clone();
  let database_path = options.cache_location.clone();
  let strategy = FileCacheStrategy::new(
    options.readonly,
    rspack_workspace::rspack_pkg_version!().to_string(),
    options.version.clone(),
    codec,
    file_system_info,
    logger.clone(),
  );
  let idle_file_cache = IdleFileCache::new(
    (base_path, database_path),
    strategy,
    logger,
    Some(options.idle_timeout),
    Some(options.idle_timeout_for_initial_store),
    Some(options.idle_timeout_after_large_changes),
  );
  let memory_cache = match options.max_memory_generations {
    MaxMemoryGenerations::Disabled => None,
    MaxMemoryGenerations::Infinity => Some(MemoryCache::new_infinite()),
    MaxMemoryGenerations::Finite(max_generations) => Some(MemoryCache::new(max_generations)),
  };

  Cache::new(memory_cache, Some(idle_file_cache))
}
