mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod db;
mod etag;
mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;
mod meta;
mod snapshot;
mod validator;

use std::sync::Arc;

pub use cache::Cache;
pub use cache_facade::{CacheFacade, ItemCacheFacade, MultiItemCache};
pub use cache_key::CacheKey;
pub use cache_value::CacheValue;
pub use etag::Etag;
pub use file_cache_strategy::FileCacheStrategy;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
pub use meta::Meta;
use rspack_fs::ReadableFileSystem;
pub use snapshot::FileSystemInfo;
pub(crate) use snapshot::{Snapshot, SnapshotValidationResult};

use crate::{CompilationLogger, CompilationLogging, CompilerOptions, cache::CacheCodec};

pub fn create_cache(
  compiler_path: String,
  compiler_options: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  compilation_logging: CompilationLogging,
) -> Cache {
  if !compiler_options.experiments.new_cache.is_enabled() {
    return Cache::new_disabled(compiler_path);
  }

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => return Cache::new_disabled(compiler_path),
    crate::CacheOptions::Memory {
      max_generations: _, /* TODO: old cache default to 1, change to 5 and pass to MemoryCache */
    } => {
      return Cache::new(compiler_path, MemoryCache::new(5), None);
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
    codec,
    file_system_info,
  ) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      return Cache::new(compiler_path, MemoryCache::default(), None);
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy, None, None, None);

  Cache::new(compiler_path, MemoryCache::default(), Some(idle_file_cache))
}
