mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod etag;
mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;
mod snapshot;

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
  compiler_options: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  compilation_logging: CompilationLogging,
) -> Cache {
  if !compiler_options.experiments.new_cache {
    return Cache::new_disabled();
  }

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => return Cache::new_disabled(),
    crate::CacheOptions::Memory { max_generations: _ } => {
      return Cache::new(MemoryCache::default(), None);
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
  let (base_path, database_path) = match &options.storage {
    crate::cache::persistent::storage::StorageOptions::FileSystem { directory } => (
      directory.clone(),
      directory.join(rspack_workspace::rspack_pkg_version!()),
    ),
  };
  let strategy = match FileCacheStrategy::new(
    base_path,
    database_path,
    options.readonly,
    codec,
    snapshot,
    build_deps,
  ) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      return Cache::new(MemoryCache::default(), None);
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy);

  Cache::new(MemoryCache::default(), Some(idle_file_cache))
}
