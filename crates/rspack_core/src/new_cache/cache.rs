use std::{sync::Arc, time::Duration};

use rspack_error::Result;
use rspack_paths::ArcPathSet;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData, snapshot::FileSystemInfo,
};

/// Cache entry point backed by memory and optional filesystem storage.
///
/// Reads follow webpack's cache stage order: memory is queried first and only
/// an unknown key falls through to the filesystem cache. Filesystem results,
/// including misses, are recorded in memory for subsequent reads.
#[derive(Debug)]
struct CacheStorage {
  memory_cache: MemoryCache,
  idle_file_cache: Option<IdleFileCache>,
}

#[derive(Debug)]
struct CacheInner {
  compiler_path: String,
  storage: Option<CacheStorage>,
  file_system_info: Option<FileSystemInfo>,
}

/// Cheaply cloneable handle to the shared cache state.
#[derive(Debug, Clone)]
pub struct Cache {
  inner: Arc<CacheInner>,
}

impl Cache {
  pub fn new(
    compiler_path: String,
    memory_cache: MemoryCache,
    idle_file_cache: Option<IdleFileCache>,
  ) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: Some(CacheStorage {
          memory_cache,
          idle_file_cache,
        }),
        file_system_info: None,
      }),
    }
  }

  pub(crate) fn new_with_file_system_info(
    compiler_path: String,
    memory_cache: MemoryCache,
    idle_file_cache: Option<IdleFileCache>,
    file_system_info: FileSystemInfo,
  ) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: Some(CacheStorage {
          memory_cache,
          idle_file_cache,
        }),
        file_system_info: Some(file_system_info),
      }),
    }
  }

  pub fn new_disabled(compiler_path: String) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: None,
        file_system_info: None,
      }),
    }
  }

  pub(crate) fn facade(&self, name: &str) -> CacheFacade {
    let mut cache_name = String::with_capacity(self.inner.compiler_path.len() + name.len());
    cache_name.push_str(&self.inner.compiler_path);
    cache_name.push_str(name);
    CacheFacade::new(self.clone(), cache_name)
  }

  pub fn get<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Result<Option<CacheValue<T>>> {
    let Some(storage) = &self.inner.storage else {
      return Ok(None);
    };
    match storage.memory_cache.get(&key, etag.as_ref()) {
      MemoryCacheGetResult::Hit(value) => Ok(Some(value)),
      MemoryCacheGetResult::Miss => Ok(None),
      MemoryCacheGetResult::NotCached => {
        let Some(file_cache) = &storage.idle_file_cache else {
          storage.memory_cache.store_miss(key);
          return Ok(None);
        };

        match file_cache.restore::<T>(key.clone(), etag.clone())? {
          Some(value) => {
            storage.memory_cache.store(key, etag, value.clone());
            Ok(Some(value))
          }
          None => {
            storage.memory_cache.store_miss(key);
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
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      storage
        .memory_cache
        .store(key.clone(), etag.clone(), value.clone());
      file_cache.store(key, etag, value)
    } else {
      storage.memory_cache.store(key, etag, value);
      Ok(())
    }
  }

  pub fn store_build_dependencies(&self, dependencies: ArcPathSet) -> Result<()> {
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_build_dependencies(dependencies)
    } else {
      Ok(())
    }
  }

  pub fn file_system_info(&self) -> Option<&FileSystemInfo> {
    self.inner.file_system_info.as_ref()
  }

  pub fn has_file_cache(&self) -> bool {
    self
      .inner
      .storage
      .as_ref()
      .is_some_and(|storage| storage.idle_file_cache.is_some())
  }

  pub fn record_build_time(&self, build_time: Duration) -> Result<()> {
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.record_build_time(build_time)
    } else {
      Ok(())
    }
  }

  pub fn begin_idle(&self) -> Result<()> {
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    storage.memory_cache.start_next_generation();
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.begin_idle()
    } else {
      Ok(())
    }
  }

  pub fn end_idle(&self) -> Result<()> {
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.end_idle()
    } else {
      Ok(())
    }
  }

  pub async fn shutdown(&self) -> Result<()> {
    if let Some(storage) = &self.inner.storage {
      storage.memory_cache.clear();
      if let Some(file_cache) = &storage.idle_file_cache {
        file_cache.shutdown().await?;
      }
    }
    Ok(())
  }
}
