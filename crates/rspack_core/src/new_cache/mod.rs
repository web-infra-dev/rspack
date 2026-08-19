mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod db;
mod etag;
mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;
mod snapshot;
mod validator;

use std::sync::Arc;

pub use cache::Cache;
pub use cache_facade::{CacheFacade, ItemCacheFacade};
pub use cache_key::CacheKey;
pub use cache_value::CacheValue;
pub use etag::Etag;
pub use file_cache_strategy::FileCacheStrategy;
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
use rspack_fs::ReadableFileSystem;

use self::snapshot::{BuildDeps, Snapshot};
use crate::{
  CompilationLogger, CompilationLogging, CompilerOptions, cache::persistent::codec::CacheCodec,
};

pub fn create_cache(
  compiler_path: String,
  compiler_options: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  compilation_logging: CompilationLogging,
) -> Cache {
  if !compiler_options.experiments.new_cache {
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

  let project_root = if options.portable {
    Some(compiler_options.context.as_path().to_path_buf())
  } else {
    None
  };
  let codec = Arc::new(CacheCodec::new(project_root));
  let snapshot = Snapshot::new(options.snapshot.clone(), input_filesystem.clone());
  let build_deps = BuildDeps::new(
    &options.build_dependencies,
    input_filesystem,
    CompilationLogger::new("rspack.newCache".to_string(), compilation_logging),
  );
  let database_paths = match &options.storage {
    crate::cache::persistent::storage::StorageOptions::FileSystem { directory } => directory
      .parent()
      .map(|base_path| (base_path.to_path_buf(), directory.clone()))
      .ok_or_else(|| {
        rspack_error::error!("Persistent cache directory must have a parent directory: {directory}")
      }),
  };
  let strategy = match database_paths.and_then(|database_paths| {
    FileCacheStrategy::new(
      database_paths,
      options.readonly,
      rspack_workspace::rspack_pkg_version!().to_string(),
      options.version.clone(),
      codec,
      snapshot,
      build_deps,
    )
  }) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      return Cache::new(compiler_path, MemoryCache::default(), None);
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy, None, None, None);

  Cache::new(compiler_path, MemoryCache::default(), Some(idle_file_cache))
}
