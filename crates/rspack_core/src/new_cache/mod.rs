mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod db;
mod etag;
mod file_cache_strategy;
mod file_dependencies;
mod idle_file_cache;
mod memory_cache;
mod meta;
mod module_build_cache;
mod snapshot;
mod validator;

use std::sync::Arc;

pub use cache::Cache;
pub use cache_facade::{CacheFacade, ItemCacheFacade, MultiItemCache};
pub use cache_key::CacheKey;
pub use cache_value::CacheValue;
pub use etag::Etag;
pub use file_cache_strategy::FileCacheStrategy;
pub(crate) use file_dependencies::FileDependencies;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
pub use meta::Meta;
pub(crate) use module_build_cache::{ModuleBuildCache, ModuleCache, ModuleCacheFactory};
use rspack_fs::ReadableFileSystem;
pub(crate) use snapshot::Snapshot;

use self::snapshot::FileSystemInfo;
use crate::{CompilationLogger, CompilationLogging, CompilerOptions, cache::CacheCodec};

#[derive(Debug)]
pub(crate) struct NewCache {
  pub(crate) cache: Cache,
  pub(crate) module_cache_factory: Option<ModuleCacheFactory>,
}

impl NewCache {
  fn disabled(compiler_path: String) -> Self {
    Self {
      cache: Cache::new_disabled(compiler_path),
      module_cache_factory: None,
    }
  }

  fn memory(compiler_path: String, memory_cache: MemoryCache, enable_module_cache: bool) -> Self {
    let cache = Cache::new(compiler_path, memory_cache, None);
    let module_cache_factory = enable_module_cache
      .then(|| ModuleCacheFactory::new(cache.clone(), Arc::new(CacheCodec::new(None))));
    Self {
      cache,
      module_cache_factory,
    }
  }
}

pub fn create_cache(
  compiler_path: String,
  compiler_options: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  compilation_logging: CompilationLogging,
) -> NewCache {
  if !compiler_options.experiments.new_cache.is_enabled() {
    return NewCache::disabled(compiler_path);
  }

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => return NewCache::disabled(compiler_path),
    crate::CacheOptions::Memory {
      max_generations: _, /* TODO: old cache default to 1, change to 5 and pass to MemoryCache */
    } => {
      return NewCache::memory(
        compiler_path,
        MemoryCache::new(5),
        compiler_options.experiments.new_cache.module,
      );
    }
    crate::CacheOptions::Persistent(options) => options,
  };

  let portable_project_root = if options.portable {
    Some(compiler_options.context.as_path().to_path_buf())
  } else {
    None
  };
  let codec = Arc::new(CacheCodec::new(portable_project_root));
  let file_system_info = FileSystemInfo::new(
    input_filesystem.clone(),
    CompilationLogger::new("rspack.FileSystemInfo".to_string(), compilation_logging),
    options.snapshot.clone(),
    compiler_options.output.hash_function,
  );
  let (base_path, database_path) = match &options.storage {
    crate::cache::StorageOptions::FileSystem { directory } => {
      let base_path = directory.parent().unwrap_or_else(|| {
        panic!("Persistent cache directory must have a parent directory: {directory}")
      });
      (base_path.to_path_buf(), directory.clone())
    }
  };
  let strategy = match FileCacheStrategy::new(
    (base_path, database_path),
    options.readonly,
    rspack_workspace::rspack_pkg_version!().to_string(),
    options.version.clone(),
    codec.clone(),
    file_system_info,
  ) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      return NewCache::memory(
        compiler_path,
        MemoryCache::default(),
        compiler_options.experiments.new_cache.module,
      );
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy, None, None, None);
  let cache = Cache::new(compiler_path, MemoryCache::default(), Some(idle_file_cache));
  let module_cache_factory = compiler_options
    .experiments
    .new_cache
    .module
    .then(|| ModuleCacheFactory::new(cache.clone(), codec));
  NewCache {
    cache,
    module_cache_factory,
  }
}
