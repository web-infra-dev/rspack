use std::{
  ops::Deref,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use rspack_paths::InternedPathSet;
use rustc_hash::FxHashMap;

use super::{
  CacheFacade, CacheInfo, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache,
  MemoryCacheGetResult, cache_value::CacheValueData, statistics::CacheStatistics,
};
use crate::InfrastructureLogger;

/// Storage shared by all compiler-scoped cache views.
#[derive(Debug)]
struct CacheStorage {
  memory_cache: Option<MemoryCache>,
  idle_file_cache: Option<IdleFileCache>,
}

/// Shared cache storage, independent of a compiler's namespace.
///
/// Reads follow webpack's cache stage order: memory is queried first and only
/// an unknown key falls through to the filesystem cache. Filesystem results,
/// including misses, are recorded in memory for subsequent reads.
#[derive(Debug)]
pub struct Cache {
  storage: CacheStorage,
  logger: Arc<InfrastructureLogger>,
  statistics: Arc<CacheStatistics>,
  closing: AtomicBool,
}

impl Cache {
  /// Creates storage from the configured memory and filesystem caches.
  pub fn new(
    memory_cache: Option<MemoryCache>,
    idle_file_cache: Option<IdleFileCache>,
    logger: Arc<InfrastructureLogger>,
  ) -> Self {
    Self {
      storage: CacheStorage {
        memory_cache,
        idle_file_cache,
      },
      logger,
      statistics: Default::default(),
      closing: AtomicBool::new(false),
    }
  }

  pub fn new_disabled(logger: Arc<InfrastructureLogger>) -> Self {
    Self::new(None, None, logger)
  }

  pub fn logger(&self) -> &InfrastructureLogger {
    &self.logger
  }

  pub fn is_closing(&self) -> bool {
    self.closing.load(Ordering::Acquire)
  }

  pub fn get<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>) -> Option<CacheValue<T>> {
    let storage = &self.storage;
    if let Some(memory_cache) = &storage.memory_cache {
      match memory_cache.get(&key, etag.as_ref()) {
        MemoryCacheGetResult::Hit(value) => return Some(value),
        MemoryCacheGetResult::Miss => return None,
        MemoryCacheGetResult::NotCached => {}
      }
    }

    let Some(file_cache) = &storage.idle_file_cache else {
      if let Some(memory_cache) = &storage.memory_cache {
        memory_cache.store_miss(key);
      }
      return None;
    };

    match file_cache.restore::<T>(key.clone(), etag.clone()) {
      Some(value) => {
        if let Some(memory_cache) = &storage.memory_cache {
          memory_cache.store(key, etag, value.clone());
        }
        Some(value)
      }
      None => {
        if let Some(memory_cache) = &storage.memory_cache {
          memory_cache.store_miss(key);
        }
        None
      }
    }
  }

  pub fn store<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>, value: CacheValue<T>) {
    let storage = &self.storage;
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.store(key.clone(), etag.clone(), value.clone());
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store(key, etag, value)
    }
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) {
    if let Some(file_cache) = &self.storage.idle_file_cache {
      file_cache.store_build_dependencies(dependencies);
    }
  }

  pub fn has_file_cache(&self) -> bool {
    self.storage.idle_file_cache.is_some()
  }

  pub fn begin_idle(&self, build_time: Duration) {
    let storage = &self.storage;
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.start_next_generation();
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.begin_idle(build_time);
    }
  }

  pub fn end_idle(&self) {
    self.statistics.reset();
    if let Some(file_cache) = &self.storage.idle_file_cache {
      file_cache.end_idle();
    }
  }

  pub async fn shutdown(&self) {
    if self.closing.swap(true, Ordering::AcqRel) {
      return;
    }
    let storage = &self.storage;
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.shutdown().await;
    }
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.clear();
    }
  }

  pub fn statistics(&self) -> &CacheStatistics {
    &self.statistics
  }
}

/// A compiler's view of shared cache storage.
#[derive(Debug, Clone)]
pub struct CompilerCache {
  cache: Arc<Cache>,
  cache_name: Arc<str>,
  compiler_path: Arc<str>,
}

impl Deref for CompilerCache {
  type Target = Cache;

  fn deref(&self) -> &Self::Target {
    &self.cache
  }
}

impl CompilerCache {
  pub fn new(cache: Arc<Cache>, cache_name: Arc<str>, compiler_path: Arc<str>) -> Self {
    Self {
      cache,
      cache_name,
      compiler_path,
    }
  }

  pub fn facade(&self, name: &str) -> CacheFacade {
    let cache_name = [self.cache_name.as_ref(), self.compiler_path.as_ref(), name].join("|");
    CacheFacade::new(Arc::clone(&self.cache), cache_name.into())
  }

  /// Returns a cache namespace shared by every compiler using this storage.
  pub fn shared_facade(&self, name: &str) -> CacheFacade {
    CacheFacade::new(Arc::clone(&self.cache), ["", name].join("|").into())
  }

  pub fn cache_info(&self) -> FxHashMap<String, CacheInfo> {
    self.cache.statistics().snapshot()
  }
}
