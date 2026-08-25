use std::{sync::Arc, time::Duration};

use rspack_error::Result;
use rspack_paths::InternedPathSet;

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData,
  module_build_cache::{ModuleBuildCache, ModuleCacheState},
  snapshot::FileSystemInfo,
};
use crate::{ValueCacheVersions, cache::CacheCodec};

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
  module_cache: Option<ModuleCacheState>,
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
    Self::new_with_storage(compiler_path, memory_cache, idle_file_cache, None)
  }

  pub(crate) fn new_with_module_cache(
    compiler_path: String,
    memory_cache: MemoryCache,
    idle_file_cache: Option<IdleFileCache>,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
  ) -> Self {
    Self::new_with_storage(
      compiler_path,
      memory_cache,
      idle_file_cache,
      Some(ModuleCacheState::new(codec, file_system_info)),
    )
  }

  fn new_with_storage(
    compiler_path: String,
    memory_cache: MemoryCache,
    idle_file_cache: Option<IdleFileCache>,
    module_cache: Option<ModuleCacheState>,
  ) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: Some(CacheStorage {
          memory_cache,
          idle_file_cache,
        }),
        module_cache,
      }),
    }
  }

  pub fn new_disabled(compiler_path: String) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: None,
        module_cache: None,
      }),
    }
  }

  pub(crate) fn module_build_cache(
    &self,
    value_cache_versions: &ValueCacheVersions,
  ) -> Option<ModuleBuildCache> {
    self
      .inner
      .module_cache
      .as_ref()
      .map(|module_cache| module_cache.build_cache(self.clone(), value_cache_versions))
  }

  /// Restore the dependency id generator before the first make pass.
  ///
  /// Cached dependencies retain their ids, so newly created dependencies must
  /// continue after the largest id stored by the persistent cache.
  pub(crate) fn restore_dependency_id(&self) {
    if let Some(module_cache) = &self.inner.module_cache {
      module_cache.restore_dependency_id(self.idle_file_cache());
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

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) -> Result<()> {
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_build_dependencies(dependencies)
    } else {
      Ok(())
    }
  }

  pub(crate) fn record_dependency_id(&self, dependency_id: u32) -> Result<()> {
    if let Some(module_cache) = &self.inner.module_cache {
      module_cache.record_dependency_id(self.idle_file_cache(), dependency_id)
    } else {
      Ok(())
    }
  }

  fn idle_file_cache(&self) -> Option<&IdleFileCache> {
    self
      .inner
      .storage
      .as_ref()
      .and_then(|storage| storage.idle_file_cache.as_ref())
  }

  pub fn has_file_cache(&self) -> bool {
    self.idle_file_cache().is_some()
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
    if let Some(module_cache) = &self.inner.module_cache {
      module_cache.clear();
    }
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
