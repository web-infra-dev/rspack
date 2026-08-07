use std::{sync::Arc, time::Duration};

use rspack_error::Result;
use rspack_paths::Utf8PathBuf;

use super::{
  CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData,
};

/// Cache entry point backed by memory and optional filesystem storage.
///
/// Reads follow webpack's cache stage order: memory is queried first and only
/// an unknown key falls through to the filesystem cache. Filesystem results,
/// including misses, are recorded in memory for subsequent reads.
#[derive(Debug, Default)]
struct CacheInner {
  memory_cache: MemoryCache,
  idle_file_cache: Option<IdleFileCache>,
}

/// Cheaply clonable handle to the shared cache state.
#[derive(Debug, Clone, Default)]
pub struct Cache {
  inner: Arc<CacheInner>,
}

impl Cache {
  pub fn new(idle_file_cache: Option<IdleFileCache>) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        memory_cache: MemoryCache::default(),
        idle_file_cache,
      }),
    }
  }

  pub async fn get<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Result<Option<CacheValue<T>>> {
    match self.inner.memory_cache.get(&key, etag.as_ref()) {
      MemoryCacheGetResult::Hit(value) => Ok(Some(value)),
      MemoryCacheGetResult::Miss => Ok(None),
      MemoryCacheGetResult::NotCached => {
        let Some(file_cache) = &self.inner.idle_file_cache else {
          self.inner.memory_cache.store_miss(key);
          return Ok(None);
        };

        match file_cache.restore::<T>(key.clone(), etag.clone()).await? {
          Some(value) => {
            self.inner.memory_cache.store(key, etag, value.clone());
            Ok(Some(value))
          }
          None => {
            self.inner.memory_cache.store_miss(key);
            Ok(None)
          }
        }
      }
    }
  }

  pub fn store<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) -> Result<()> {
    if let Some(file_cache) = &self.inner.idle_file_cache {
      self
        .inner
        .memory_cache
        .store(key.clone(), etag.clone(), value.clone());
      file_cache.store(key, etag, value)
    } else {
      self.inner.memory_cache.store(key, etag, value);
      Ok(())
    }
  }

  pub fn store_build_dependencies(&self, dependencies: Vec<Utf8PathBuf>) -> Result<()> {
    if let Some(file_cache) = &self.inner.idle_file_cache {
      file_cache.store_build_dependencies(dependencies)
    } else {
      Ok(())
    }
  }

  pub(crate) fn has_file_cache(&self) -> bool {
    self.inner.idle_file_cache.is_some()
  }

  pub fn record_build_time(&self, build_time: Duration) -> Result<()> {
    if let Some(file_cache) = &self.inner.idle_file_cache {
      file_cache.record_build_time(build_time)
    } else {
      Ok(())
    }
  }

  pub fn begin_idle(&self) -> Result<()> {
    if let Some(file_cache) = &self.inner.idle_file_cache {
      file_cache.begin_idle()
    } else {
      Ok(())
    }
  }

  pub fn end_idle(&self) -> Result<()> {
    if let Some(file_cache) = &self.inner.idle_file_cache {
      file_cache.end_idle()
    } else {
      Ok(())
    }
  }

  pub async fn shutdown(&self) -> Result<()> {
    self.inner.memory_cache.clear();
    if let Some(file_cache) = &self.inner.idle_file_cache {
      file_cache.shutdown().await?;
    }
    Ok(())
  }
}
