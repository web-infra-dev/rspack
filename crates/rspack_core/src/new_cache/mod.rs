mod cache;
mod cache_facade;
mod cache_key;
mod cache_value;
mod db;
mod etag;
mod file_cache_strategy;
mod idle_file_cache;
mod memory_cache;
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
pub use idle_file_cache::IdleFileCache;
pub use memory_cache::{MemoryCache, MemoryCacheGetResult};
pub use module_build_cache::ModuleBuildCache;
use rspack_fs::ReadableFileSystem;

use self::snapshot::{BuildDeps, FileSystemInfo};
use crate::{
  CompilationLogger, CompilationLogging, CompilerOptions,
  cache::{CacheCodec, SnapshotOptions},
};

const NEW_CACHE_DIRECTORY: &str = "new-cache";
const DATABASE_DIRECTORY: &str = "db";

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
      let snapshot_options = SnapshotOptions::default();
      let strategy = snapshot_options.dependencies_strategy();
      let file_system_info = FileSystemInfo::new(
        input_filesystem,
        snapshot_options,
        compiler_options.output.hash_function,
      );
      return Cache::new(
        compiler_path,
        MemoryCache::new(5),
        None,
        file_system_info,
        Arc::new(CacheCodec::new(None)),
        strategy,
      );
    }
    crate::CacheOptions::Persistent(options) => options,
  };

  let project_root = if options.portable {
    Some(compiler_options.context.as_path().to_path_buf())
  } else {
    None
  };
  let codec = Arc::new(CacheCodec::new(project_root));
  let snapshot_strategy = options.snapshot.dependencies_strategy();
  let file_system_info = FileSystemInfo::new(
    input_filesystem.clone(),
    options.snapshot.clone(),
    compiler_options.output.hash_function,
  );
  let build_deps = BuildDeps::new(
    &options.build_dependencies,
    input_filesystem,
    CompilationLogger::new("rspack.newCache".to_string(), compilation_logging),
  );
  // The database owns its directory: resetting it moves the whole directory
  // away, so it must not be shared with the legacy cache, which stores its
  // packs next to it under the configured storage location.
  let (base_path, database_path) = match &options.storage {
    crate::cache::StorageOptions::FileSystem { directory } => {
      let base_path = directory.join(NEW_CACHE_DIRECTORY);
      let database_path = base_path.join(DATABASE_DIRECTORY);
      (base_path, database_path)
    }
  };
  let strategy = match FileCacheStrategy::new(
    (base_path, database_path),
    options.readonly,
    rspack_workspace::rspack_pkg_version!().to_string(),
    options.version.clone(),
    codec.clone(),
    file_system_info.clone(),
    build_deps,
  ) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      return Cache::new(
        compiler_path,
        MemoryCache::default(),
        None,
        file_system_info,
        codec,
        snapshot_strategy,
      );
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy, None, None, None);

  Cache::new(
    compiler_path,
    MemoryCache::default(),
    Some(idle_file_cache),
    file_system_info,
    codec,
    snapshot_strategy,
  )
}
