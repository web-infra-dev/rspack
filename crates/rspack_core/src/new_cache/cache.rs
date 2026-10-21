use std::{
  ops::Deref,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use rspack_paths::InternedPathSet;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData,
};
use crate::PersistentCacheState;

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
  storage: Option<Arc<CacheStorage>>,
  read_failed: AtomicBool,
}

impl Cache {
  pub(crate) fn initial_validation(&self) -> PersistentCacheState {
    self
      .storage
      .as_ref()
      .and_then(|storage| storage.idle_file_cache.as_ref())
      .map_or(
        PersistentCacheState::Unknown,
        IdleFileCache::initial_validation,
      )
  }

  /// Creates storage from the configured memory and filesystem caches.
  pub fn new(memory_cache: Option<MemoryCache>, idle_file_cache: Option<IdleFileCache>) -> Self {
    let storage = if memory_cache.is_some() || idle_file_cache.is_some() {
      Some(Arc::new(CacheStorage {
        memory_cache,
        idle_file_cache,
      }))
    } else {
      None
    };
    Self {
      storage,
      read_failed: Default::default(),
    }
  }

  pub(crate) fn read_failed(&self) -> bool {
    self.read_failed.load(Ordering::Relaxed)
  }

  pub fn new_disabled() -> Self {
    Self::new(None, None)
  }

  pub fn get<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>) -> Option<CacheValue<T>> {
    let Some(storage) = &self.storage else {
      return None;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      match memory_cache.get(&key, etag.as_ref()) {
        MemoryCacheGetResult::Hit(value) => return Some(value),
        MemoryCacheGetResult::Miss => return None,
        MemoryCacheGetResult::NotCached => {}
      }
    }

    let restored = storage.idle_file_cache.as_ref().and_then(|file_cache| {
      file_cache
        .restore::<T>(&key, etag.as_ref())
        .unwrap_or_else(|_| {
          self.read_failed.store(true, Ordering::Relaxed);
          None
        })
    });
    if let Some(memory_cache) = &storage.memory_cache {
      match &restored {
        Some(value) => memory_cache.store(key, etag, value.clone()),
        None => memory_cache.store_miss(key),
      }
    }
    restored
  }

  pub fn store<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>, value: CacheValue<T>) {
    let Some(storage) = &self.storage else {
      return;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.store(key.clone(), etag.clone(), value.clone());
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store(key, etag, value)
    }
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) {
    let Some(storage) = &self.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_build_dependencies(dependencies);
    }
  }

  pub fn has_file_cache(&self) -> bool {
    self
      .storage
      .as_ref()
      .is_some_and(|storage| storage.idle_file_cache.is_some())
  }

  pub fn begin_idle(&self, build_time: Duration) {
    let Some(storage) = &self.storage else {
      return;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.start_next_generation();
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.begin_idle(build_time);
    }
  }

  pub fn end_idle(&self) {
    let Some(storage) = &self.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.end_idle();
    }
  }

  pub async fn shutdown(&self) {
    let Some(storage) = &self.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.shutdown().await;
    }
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.clear();
    }
  }
}

/// A compiler's view of shared cache storage.
#[derive(Debug, Clone)]
pub struct CompilerCache {
  cache: Arc<Cache>,
  compiler_path: Arc<str>,
}

impl Deref for CompilerCache {
  type Target = Cache;

  fn deref(&self) -> &Self::Target {
    &self.cache
  }
}

impl CompilerCache {
  pub fn new(cache: Arc<Cache>, compiler_path: Arc<str>) -> Self {
    Self {
      cache,
      compiler_path,
    }
  }

  pub(crate) fn begin_compilation(&mut self) {
    if self.has_file_cache() {
      // Child compilers share entries, but a read failure belongs to its caller.
      self.cache = Arc::new(Cache {
        storage: self.cache.storage.clone(),
        read_failed: Default::default(),
      });
    }
  }

  pub fn facade(&self, name: &str) -> CacheFacade {
    let cache_name = [self.compiler_path.as_ref(), name].join("|");
    CacheFacade::new(Arc::clone(&self.cache), cache_name)
  }
}
