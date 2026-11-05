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
use idle_file_cache::FileCacheWorker;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
use parking_lot::Mutex;
use rspack_fs::ReadableFileSystem;
use rspack_paths::{InternedPath, InternedPathMap};
pub use snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult};
pub use statistics::{CacheInfo, CacheRecord};

use crate::{
  CompilerOptions, InfrastructureLogSink, InfrastructureLogger, MaxMemoryGenerations,
  cache::CacheCodec,
};

struct SharedCache {
  owner: Weak<CacheOwner>,
  worker: FileCacheWorker,
}

static SHARED_PERSISTENT_CACHES: LazyLock<Mutex<InternedPathMap<SharedCache>>> =
  LazyLock::new(|| Mutex::new(InternedPathMap::default()));

/// Shared by compiler roots. Compilation and cache facade references only retain the storage.
#[derive(Debug)]
pub struct CacheOwner {
  cache: Arc<Cache>,
}

impl CacheOwner {
  pub fn cache(&self) -> &Arc<Cache> {
    &self.cache
  }

  pub async fn shutdown(self: Arc<Self>) {
    if let Some(owner) = Arc::into_inner(self) {
      let worker = owner.cache.file_cache().map(IdleFileCache::worker);
      drop(owner);
      if let Some(worker) = worker {
        worker.closed().await;
      }
    }
  }
}

impl Drop for CacheOwner {
  fn drop(&mut self) {
    self.cache.start_shutdown();
    if let Some(file_cache) = self.cache.file_cache() {
      remove_closed_cache(file_cache.storage_dir());
    }
  }
}

/// Creates cache storage independently of a compiler's namespace.
pub fn create_cache(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
) -> Arc<CacheOwner> {
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
    return create_cache_owner(
      compiler_options,
      input_filesystem,
      infrastructure_log_sink,
      None,
    );
  };

  let mut caches = SHARED_PERSISTENT_CACHES.lock();
  let previous = caches.get(&storage_dir);
  if let Some(owner) = previous.and_then(|entry| entry.owner.upgrade()) {
    return owner;
  }
  // The constructor can run inside Tokio. A replacement worker waits for the
  // previous database to close on its own thread, outside the registry lock.
  let owner = create_cache_owner(
    compiler_options,
    input_filesystem,
    infrastructure_log_sink,
    previous.map(|entry| entry.worker.clone()),
  );
  let worker = owner
    .cache
    .file_cache()
    .expect("persistent cache should have a file cache")
    .worker();
  caches.insert(
    storage_dir,
    SharedCache {
      owner: Arc::downgrade(&owner),
      worker,
    },
  );
  owner
}

fn create_cache_owner(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
  previous_worker: Option<FileCacheWorker>,
) -> Arc<CacheOwner> {
  Arc::new(CacheOwner {
    cache: Arc::new(create_cache_storage(
      compiler_options,
      input_filesystem,
      infrastructure_log_sink,
      previous_worker,
    )),
  })
}

fn create_cache_storage(
  compiler_options: &CompilerOptions,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  infrastructure_log_sink: Arc<dyn InfrastructureLogSink>,
  previous_worker: Option<FileCacheWorker>,
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
    previous_worker,
  );
  let memory_cache = match options.max_memory_generations {
    MaxMemoryGenerations::Disabled => None,
    MaxMemoryGenerations::Infinity => Some(MemoryCache::new_infinite()),
    MaxMemoryGenerations::Finite(max_generations) => Some(MemoryCache::new(max_generations)),
  };

  Cache::new(memory_cache, Some(idle_file_cache), logger)
}

fn remove_closed_cache(storage_dir: &InternedPath) {
  let mut caches = SHARED_PERSISTENT_CACHES.lock();
  if caches
    .get(storage_dir)
    .is_some_and(|entry| entry.owner.strong_count() == 0 && entry.worker.is_closed())
  {
    caches.remove(storage_dir);
  }
}
