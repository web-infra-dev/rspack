use std::{ops::Deref, sync::Arc, time::Duration};

use rspack_paths::InternedPathSet;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData,
};

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
  storage: Option<CacheStorage>,
  readonly_from: Option<IdleFileCache>,
  module_profile: Option<String>,
  loader_profile: Option<String>,
}

impl Cache {
  /// Creates storage from the configured memory and filesystem caches.
  pub fn new(memory_cache: Option<MemoryCache>, idle_file_cache: Option<IdleFileCache>) -> Self {
    let storage = if memory_cache.is_some() || idle_file_cache.is_some() {
      Some(CacheStorage {
        memory_cache,
        idle_file_cache,
      })
    } else {
      None
    };
    Self {
      storage,
      readonly_from: None,
      module_profile: None,
      loader_profile: None,
    }
  }

  pub(super) fn with_reuse(
    mut self,
    readonly_from: Option<IdleFileCache>,
    module_profile: Option<String>,
    loader_profile: Option<String>,
  ) -> Self {
    self.readonly_from = readonly_from;
    self.module_profile = module_profile;
    self.loader_profile = loader_profile;
    self
  }

  pub(crate) fn module_profile(&self) -> Option<&str> {
    self.module_profile.as_deref()
  }

  pub(crate) fn loader_profile(&self) -> Option<&str> {
    self.loader_profile.as_deref()
  }

  /// Only make-stage callers may consult this source. Promotion is deferred until
  /// the caller has validated the entry's file and value dependencies.
  pub(crate) fn get_imported<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Etag,
  ) -> Option<CacheValue<T>> {
    let source = self.readonly_from.as_ref()?;
    let value = source.restore(key, Some(etag))?;
    // Promotion must preserve invalidation of the producer's configuration and
    // plugins, even when subsequent builds only read the consumer's own cache.
    self.store_build_dependencies(source.build_dependencies());
    Some(value)
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
    if let Some(source) = &self.readonly_from {
      source.shutdown().await;
    }
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

  pub fn facade(&self, name: &str) -> CacheFacade {
    let cache_name = [self.compiler_path.as_ref(), name].join("|");
    CacheFacade::new(
      Arc::clone(&self.cache),
      cache_name,
      self.compiler_path.is_empty(),
    )
  }
}
