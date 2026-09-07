use std::{any::Any, sync::Arc, time::Duration};

use rayon::prelude::*;
use rspack_cacheable::{__private::rkyv::Serialize, Serializer};
use rspack_error::Result;
use rspack_paths::InternedPathSet;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult, Meta,
  cache_value::CacheValueData,
};

/// Cache entry point backed by memory and optional filesystem storage.
///
/// Reads follow webpack's cache stage order: memory is queried first and only
/// an unknown key falls through to the filesystem cache. Filesystem results,
/// including misses, are recorded in memory for subsequent reads.
#[derive(Debug)]
struct CacheStorage {
  memory_cache: Option<MemoryCache>,
  idle_file_cache: Option<IdleFileCache>,
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

  pub(crate) fn get_memory<T: Any + Send + Sync>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> MemoryCacheGetResult<T> {
    self
      .inner
      .storage
      .as_ref()
      .and_then(|storage| storage.memory_cache.as_ref())
      .map_or(MemoryCacheGetResult::NotCached, |cache| {
        cache.get(&key, etag.as_ref())
      })
  }

  pub(crate) fn store_memory<T: Any + Send + Sync>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) {
    if let Some(cache) = self
      .inner
      .storage
      .as_ref()
      .and_then(|storage| storage.memory_cache.as_ref())
    {
      cache.store(key, etag, value);
    }
  }

  pub(crate) fn restore_owned<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Option<T> {
    self
      .inner
      .storage
      .as_ref()?
      .idle_file_cache
      .as_ref()?
      .restore_owned(key, etag)
  }

  pub(crate) fn store_borrowed<T: for<'a> Serialize<Serializer<'a>> + Send>(
    &self,
    entries: impl ParallelIterator<Item = (CacheKey, Option<Etag>, T)>,
  ) {
    if let Some(cache) = self
      .inner
      .storage
      .as_ref()
      .and_then(|storage| storage.idle_file_cache.as_ref())
    {
      cache.store_borrowed(entries);
    }
  }

  pub(crate) fn has_memory_cache(&self) -> bool {
    self
      .inner
      .storage
      .as_ref()
      .is_some_and(|storage| storage.memory_cache.is_some())
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
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.shutdown().await;
    }
  }
}
