use std::{
  any::Any,
  sync::{Arc, Weak},
  time::Duration,
};

use rspack_error::Result;
use rspack_paths::InternedPathSet;
use rspack_util::fx_hash::FxDashMap;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult, Meta,
  cache_value::CacheValueData,
};
use crate::cache::CacheCodec;

/// Cache entry point backed by memory and optional filesystem storage.
///
/// Reads follow webpack's cache stage order: memory is queried first and only
/// an unknown key falls through to the filesystem cache. Filesystem results,
/// including misses, are recorded in memory for subsequent reads.
#[derive(Debug)]
struct CacheStorage {
  memory_cache: Option<MemoryCache>,
  idle_file_cache: Option<IdleFileCache>,
  // Evicting a strong cache entry must not deserialize a second live object
  // while a graph still owns the original access lease.
  // Only a filesystem backend needs this index; memory-only caching does not
  // allocate or maintain a second map of module handles.
  live_values: Option<FxDashMap<CacheKey, Weak<dyn Any + Send + Sync>>>,
}

#[derive(Debug)]
struct CacheInner {
  compiler_path: String,
  storage: Option<CacheStorage>,
}

/// Cheaply cloneable handle to the shared cache state.
#[derive(Debug, Clone)]
pub struct Cache {
  inner: Arc<CacheInner>,
}

impl Cache {
  pub fn new(
    compiler_path: String,
    memory_cache: Option<MemoryCache>,
    idle_file_cache: Option<IdleFileCache>,
  ) -> Self {
    let storage = if memory_cache.is_some() || idle_file_cache.is_some() {
      Some(CacheStorage {
        live_values: idle_file_cache.as_ref().map(|_| Default::default()),
        memory_cache,
        idle_file_cache,
      })
    } else {
      None
    };
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage,
      }),
    }
  }

  pub fn new_disabled(compiler_path: String) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: None,
      }),
    }
  }

  pub(crate) fn facade(&self, name: &str) -> CacheFacade {
    let mut cache_name = String::with_capacity(self.inner.compiler_path.len() + name.len());
    cache_name.push_str(&self.inner.compiler_path);
    cache_name.push_str(name);
    CacheFacade::new(self.clone(), cache_name)
  }

  pub fn get<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>) -> Option<CacheValue<T>> {
    let Some(storage) = &self.inner.storage else {
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
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.store(key.clone(), etag.clone(), value.clone());
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store(key, etag, value)
    }
  }

  pub(crate) fn get_live<T: Any + Send + Sync>(
    &self,
    key: CacheKey,
    decode: impl FnOnce(&[u8], &CacheCodec) -> Result<CacheValue<T>>,
  ) -> Option<CacheValue<T>> {
    let storage = self.inner.storage.as_ref()?;
    if let Some(memory) = &storage.memory_cache {
      match memory.get(&key, None) {
        MemoryCacheGetResult::Hit(value) => return Some(value),
        MemoryCacheGetResult::Miss => return None,
        MemoryCacheGetResult::NotCached => {}
      }
    }
    if let Some(value) = storage
      .live_values
      .as_ref()
      .and_then(|values| values.get(&key).and_then(|value| value.upgrade()))
    {
      return Arc::downcast::<T>(value).ok().map(CacheValue::from);
    }
    let result = storage
      .idle_file_cache
      .as_ref()
      .and_then(|file| file.restore_live(key.clone(), decode));
    if let Some(value) = &result
      && let Some(live_values) = &storage.live_values
    {
      let erased: Arc<dyn Any + Send + Sync> = value.as_arc().clone();
      live_values.insert(key.clone(), Arc::downgrade(&erased));
    }
    if let Some(memory) = &storage.memory_cache {
      if let Some(value) = &result {
        memory.store(key, None, value.clone());
      } else {
        memory.store_miss(key);
      }
    }
    result
  }

  pub(crate) fn store_live<T: Any + Send + Sync>(&self, key: CacheKey, value: CacheValue<T>) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(live_values) = &storage.live_values {
      let erased: Arc<dyn Any + Send + Sync> = value.as_arc().clone();
      live_values.insert(key.clone(), Arc::downgrade(&erased));
    }
    if let Some(memory) = &storage.memory_cache {
      memory.store(key.clone(), None, value.clone());
    }
    if let Some(file) = &storage.idle_file_cache {
      file.store_live(key, value);
    }
  }

  pub(crate) fn encode_live<T: Any + Send + Sync>(
    &self,
    key: CacheKey,
    value: CacheValue<T>,
    encode: impl FnOnce(&CacheCodec) -> Result<Vec<u8>>,
  ) {
    if let Some(file) = self
      .inner
      .storage
      .as_ref()
      .and_then(|storage| storage.idle_file_cache.as_ref())
    {
      file.encode_live(key, value, encode);
    }
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_build_dependencies(dependencies);
    }
  }

  pub fn store_meta(&self, meta: Meta) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_meta(meta);
    }
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    let Some(storage) = &self.inner.storage else {
      return Ok(None);
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.restore_meta()
    } else {
      Ok(None)
    }
  }

  pub fn has_file_cache(&self) -> bool {
    self
      .inner
      .storage
      .as_ref()
      .is_some_and(|storage| storage.idle_file_cache.is_some())
  }

  pub fn begin_idle(&self, build_time: Duration) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.start_next_generation();
    }
    if let Some(live_values) = &storage.live_values {
      live_values.retain(|_, value| value.strong_count() != 0);
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.begin_idle(build_time);
    }
  }

  pub fn end_idle(&self) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.end_idle();
    }
  }

  pub async fn shutdown(&self) {
    let Some(storage) = &self.inner.storage else {
      return;
    };
    if let Some(memory_cache) = &storage.memory_cache {
      memory_cache.clear();
    }
    if let Some(live_values) = &storage.live_values {
      live_values.clear();
    }
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.shutdown().await;
    }
  }
}
