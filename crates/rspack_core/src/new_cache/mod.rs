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
pub(crate) use module_build_cache::{ModuleBuildCache, ModuleCache, ModuleCacheFactory};
use rspack_fs::ReadableFileSystem;

use self::snapshot::{BuildDeps, FileSystemInfo};
use crate::{CompilationLogger, CompilationLogging, CompilerOptions, cache::CacheCodec};

pub(crate) struct NewCache {
  pub(crate) cache: Cache,
  pub(crate) module_cache_factory: Option<ModuleCacheFactory>,
}

pub(crate) fn create_cache(
  compiler_path: String,
  compiler_options: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  compilation_logging: CompilationLogging,
) -> NewCache {
  if !compiler_options.experiments.new_cache.is_enabled() {
    return NewCache {
      cache: Cache::new_disabled(compiler_path),
      module_cache_factory: None,
    };
  }
  let module_cache_enabled = compiler_options.experiments.new_cache.module;

  let options = match &compiler_options.cache {
    crate::CacheOptions::Disabled => {
      return NewCache {
        cache: Cache::new_disabled(compiler_path),
        module_cache_factory: None,
      };
    }
    crate::CacheOptions::Memory {
      max_generations: _, /* TODO: old cache default to 1, change to 5 and pass to MemoryCache */
    } => {
      let module_cache_codec = module_cache_enabled.then(|| Arc::new(CacheCodec::new(None)));
      return assemble_cache(compiler_path, MemoryCache::new(5), None, module_cache_codec);
    }
    crate::CacheOptions::Persistent(options) => options,
  };

  let project_root = if options.portable {
    Some(compiler_options.context.as_path().to_path_buf())
  } else {
    None
  };
  let codec = Arc::new(CacheCodec::new(project_root));
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
    build_deps,
  ) {
    Ok(strategy) => strategy,
    Err(error) => {
      tracing::warn!("Opening persistent cache database failed: {error}");
      let module_cache_codec = module_cache_enabled.then_some(codec);
      return assemble_cache(
        compiler_path,
        MemoryCache::default(),
        None,
        module_cache_codec,
      );
    }
  };
  let idle_file_cache = IdleFileCache::new(strategy, None, None, None);
  let module_cache_codec = module_cache_enabled.then_some(codec);
  assemble_cache(
    compiler_path,
    MemoryCache::default(),
    Some(idle_file_cache),
    module_cache_codec,
  )
}

fn assemble_cache(
  compiler_path: String,
  memory_cache: MemoryCache,
  idle_file_cache: Option<IdleFileCache>,
  module_cache_codec: Option<Arc<CacheCodec>>,
) -> NewCache {
  let cache = Cache::new(compiler_path, memory_cache, idle_file_cache.clone());
  let module_cache_factory =
    module_cache_codec.map(|codec| ModuleCacheFactory::new(cache.clone(), codec, idle_file_cache));
  NewCache {
    cache,
    module_cache_factory,
  }
}
