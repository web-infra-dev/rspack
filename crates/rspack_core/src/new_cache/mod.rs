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
mod statistics;
mod validator;

use std::sync::{Arc, LazyLock, Weak};

pub use cache::{Cache, CompilerCache};
pub use cache_facade::{CacheFacade, ItemCacheFacade, MultiItemCache};
pub use cache_key::CacheKey;
pub use cache_value::CacheValue;
pub use etag::Etag;
pub use file_cache_strategy::FileCacheStrategy;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
use rspack_fs::ReadableFileSystem;
use rspack_paths::{InternedPath, InternedPathMap};
pub use snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult};
pub use statistics::{CacheInfo, CacheRecord};
use tokio::sync::Mutex;

use crate::{
  CompilerOptions, InfrastructureLogSink, InfrastructureLogger, MaxMemoryGenerations,
  cache::CacheCodec,
};

static SHARED_PERSISTENT_CACHES: LazyLock<Mutex<InternedPathMap<Weak<Cache>>>> =
  LazyLock::new(|| Mutex::new(InternedPathMap::default()));

/// Creates cache storage independently of a compiler's namespace.
pub fn create_cache(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
) -> Arc<Cache> {
  let storage_dir = match (
    compiler_options.experiments.new_cache.is_enabled(),
    &compiler_options.cache,
  ) {
    (true, crate::CacheOptions::Persistent(options)) => match &options.storage {
      crate::StorageOptions::FileSystem { location, .. } => {
        Some(InternedPath::from(location.as_path()))
      }
    },
    _ => None,
  };

  let Some(storage_dir) = storage_dir else {
    return Arc::new(create_cache_storage(
      compiler_options,
      input_filesystem,
      infrastructure_log_sink,
    ));
  };

  let mut caches = SHARED_PERSISTENT_CACHES.blocking_lock();
  if let Some(cache) = caches.get(&storage_dir).and_then(Weak::upgrade) {
    return cache;
  }
  let cache = Arc::new(create_cache_storage(
    compiler_options,
    input_filesystem,
    infrastructure_log_sink,
  ));
  caches.insert(storage_dir, Arc::downgrade(&cache));
  cache
}

fn create_cache_storage(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
) -> Cache {
  let logger = Arc::new(InfrastructureLogger::new(
    "rspack.cache",
    infrastructure_log_sink,
  ));
  if !compiler_options.experiments.new_cache.is_enabled() {
    return Cache::new_disabled(logger);
  }

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => {
      return Cache::new_disabled(logger);
    }
    crate::CacheOptions::Memory {
      max_generations: _, /* TODO: old cache default to 1, change to 5 and pass to MemoryCache */
      ..
    } => {
      return Cache::new(Some(MemoryCache::new(5)), None, logger);
    }
    crate::CacheOptions::Persistent(options) => options,
  };

  let codec = Arc::new(CacheCodec::new(None));
  let file_system_info = FileSystemInfo::new(
    input_filesystem,
    logger.get_child("rspack.FileSystemInfo"),
    compiler_options.snapshot.clone(),
    compiler_options.context.clone(),
    compiler_options.output.hash_function,
  );
  let (base_path, database_path) = match &options.storage {
    crate::StorageOptions::FileSystem {
      directory,
      location,
    } => (directory.clone(), location.clone()),
  };
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
    logger.clone(),
    None,
    None,
    None,
  );
  let memory_cache = match options.max_memory_generations {
    MaxMemoryGenerations::Disabled => None,
    MaxMemoryGenerations::Infinity => Some(MemoryCache::new_infinite()),
    MaxMemoryGenerations::Finite(max_generations) => Some(MemoryCache::new(max_generations)),
  };

  Cache::new(memory_cache, Some(idle_file_cache), logger)
}

pub async fn shutdown_cache(cache: Arc<Cache>) {
  let Some(storage_dir) = cache.storage_dir() else {
    cache.shutdown().await;
    return;
  };

  let mut caches = SHARED_PERSISTENT_CACHES.lock().await;
  let is_current = caches
    .get(&storage_dir)
    .and_then(Weak::upgrade)
    .is_some_and(|current| Arc::ptr_eq(&current, &cache));
  cache.shutdown().await;
  if is_current {
    caches.remove(&storage_dir);
  }
}
