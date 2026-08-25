use std::{
  sync::{Arc, OnceLock},
  time::Duration,
};

use rspack_error::Result;
use rspack_paths::InternedPathSet;
use rspack_tasks::{get_current_dependency_id, set_current_dependency_id};

use super::{
  CacheFacade, CacheKey, CacheValue, Etag, IdleFileCache, MemoryCache, MemoryCacheGetResult,
  cache_value::CacheValueData,
  module_build_cache::ModuleBuildCache,
  snapshot::{FileSystemInfo, Snapshot, SnapshotValidationResult},
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
  codec: Option<Arc<CacheCodec>>,
  file_system_info: Option<FileSystemInfo>,
  dependency_id_restored: OnceLock<()>,
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
        codec: None,
        file_system_info: None,
        dependency_id_restored: OnceLock::new(),
      }),
    }
  }

  pub(crate) fn new_with_module_cache(
    compiler_path: String,
    memory_cache: MemoryCache,
    idle_file_cache: Option<IdleFileCache>,
    codec: Arc<CacheCodec>,
    file_system_info: FileSystemInfo,
  ) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: Some(CacheStorage {
          memory_cache,
          idle_file_cache,
        }),
        codec: Some(codec),
        file_system_info: Some(file_system_info),
        dependency_id_restored: OnceLock::new(),
      }),
    }
  }

  pub fn new_disabled(compiler_path: String) -> Self {
    Self {
      inner: Arc::new(CacheInner {
        compiler_path,
        storage: None,
        codec: None,
        file_system_info: None,
        dependency_id_restored: OnceLock::new(),
      }),
    }
  }

  pub(crate) fn is_module_cache_enabled(&self) -> bool {
    self.inner.storage.is_some()
      && self.inner.codec.is_some()
      && self.inner.file_system_info.is_some()
  }

  pub(crate) fn module_build_cache(
    &self,
    value_cache_versions: Arc<ValueCacheVersions>,
  ) -> Option<ModuleBuildCache> {
    self
      .is_module_cache_enabled()
      .then(|| ModuleBuildCache::new(self.clone(), value_cache_versions))
  }

  pub(crate) fn codec(&self) -> Option<&CacheCodec> {
    self.inner.codec.as_deref()
  }

  pub(crate) fn file_system_info(&self) -> Option<&FileSystemInfo> {
    self.inner.file_system_info.as_ref()
  }

  pub(crate) async fn create_module_snapshot(
    &self,
    start_time: u64,
    file_dependencies: &InternedPathSet,
    context_dependencies: &InternedPathSet,
    missing_dependencies: &InternedPathSet,
    build_dependencies: &InternedPathSet,
  ) -> Result<Option<Snapshot>> {
    let Some(file_system_info) = self.file_system_info() else {
      return Ok(None);
    };
    let mut files = file_dependencies.clone();
    files.extend(build_dependencies.iter().cloned());
    file_system_info
      .create_snapshot(
        Some(start_time),
        &files,
        context_dependencies,
        missing_dependencies,
        file_system_info.module_strategy(),
      )
      .await
      .map(Some)
  }

  pub(crate) async fn check_module_snapshot_valid(&self, snapshot: &Snapshot) -> Result<bool> {
    let Some(file_system_info) = self.file_system_info() else {
      return Ok(false);
    };
    Ok(matches!(
      file_system_info.check_snapshot_valid(snapshot).await?,
      SnapshotValidationResult::Valid
    ))
  }

  /// Restore the dependency id generator before the first make pass.
  ///
  /// Cached dependencies retain their ids, so newly created dependencies must
  /// continue after the largest id stored by the persistent cache.
  pub(crate) fn restore_dependency_id(&self) {
    if !self.is_module_cache_enabled() {
      return;
    }
    self.inner.dependency_id_restored.get_or_init(|| {
      let Some(file_cache) = self
        .inner
        .storage
        .as_ref()
        .and_then(|storage| storage.idle_file_cache.as_ref())
      else {
        return;
      };

      let dependency_id = match file_cache.restore_dependency_id() {
        Ok(dependency_id) => dependency_id,
        Err(error) => {
          tracing::warn!("Restoring new cache dependency id failed: {error}");
          None
        }
      };
      if let Some(dependency_id) = dependency_id {
        let current = get_current_dependency_id();
        if current < dependency_id {
          set_current_dependency_id(dependency_id);
        }
      }
    });
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
    if !self.is_module_cache_enabled() {
      return Ok(());
    }
    let Some(storage) = &self.inner.storage else {
      return Ok(());
    };
    if let Some(file_cache) = &storage.idle_file_cache {
      file_cache.store_dependency_id(dependency_id)
    } else {
      Ok(())
    }
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
    if let Some(file_system_info) = &self.inner.file_system_info {
      file_system_info.clear();
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
