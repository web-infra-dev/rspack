mod disable;
mod memory;
mod mixed;
pub mod persistent;

use std::{fmt::Debug, path::PathBuf, sync::Arc};

use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};
use rspack_paths::Utf8PathBuf;

use self::{
  disable::DisableCache, memory::MemoryCache, mixed::MixedCache, persistent::PersistentCache,
};
use crate::{CacheOptions, Compilation, CompilationLogging, CompilerOptions, MaxMemoryGenerations};

/// Cache trait
///
/// The cache trait provides lifecycle methods for restoring and saving cached
/// build results. Incremental artifacts are managed separately by the compiler.
///
/// ### Why not define it as a hook directly
/// * The design of cache is different from webpack.
/// * Hook is relatively complex.
/// * This API does not need to cooperate with the js side.
///
/// We can consider change to Hook when we need to open the API to js side.
#[async_trait::async_trait]
pub trait Cache: Debug + Send + Sync {
  /// before compile return is_hot_start
  async fn before_compile(&mut self, _compilation: &mut Compilation) -> bool {
    false
  }
  async fn after_compile(&mut self, _compilation: &Compilation) {}

  // BUILD_MODULE_GRAPH hooks
  async fn before_build_module_graph(&mut self, _compilation: &mut Compilation) {}
  async fn after_build_module_graph(&mut self, _compilation: &Compilation) {}

  // PROCESS_ASSETS hooks
  async fn before_process_assets(&mut self, _compilation: &mut Compilation) {}
  async fn after_process_assets(&mut self, _compilation: &Compilation) {}

  /// Move process-local cache entries out of the completed compilation before
  /// it becomes the previous incremental compilation.
  fn store_hot_cache(&mut self, _compilation: &mut Compilation) {}

  /// Shuts down the cache, flushing all pending background storage writes to completion.
  async fn close(&self) {}
}

pub fn create_cache(
  compiler_path: &str,
  compiler_option: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
  compilation_logging: CompilationLogging,
) -> Box<dyn Cache> {
  if compiler_option.experiments.new_cache.is_enabled() {
    return Box::new(DisableCache);
  }

  match &compiler_option.cache {
    CacheOptions::Disabled | CacheOptions::FileSystem(_) => Box::new(DisableCache),
    CacheOptions::Memory { .. } => Box::<MemoryCache>::default(),
    CacheOptions::Persistent(option) => {
      let persistent = PersistentCache::new(
        compiler_path,
        option,
        compiler_option.clone(),
        input_filesystem,
        intermediate_filesystem,
        compilation_logging,
      );
      Box::new(MixedCache::new(persistent))
    }
  }
}

pub type BuildDepsOptions = Vec<PathBuf>;

/// Storage options for the legacy persistent cache.
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub enum StorageOptions {
  FileSystem {
    #[cacheable(with=As<PortablePath>)]
    directory: Utf8PathBuf,
  },
}

/// Options for the legacy persistent cache.
#[derive(Debug, Clone)]
pub struct PersistentCacheOptions {
  pub build_dependencies: BuildDepsOptions,
  pub version: String,
  pub storage: StorageOptions,
  pub portable: bool,
  pub readonly: bool,
  /// Filesystem cache max age in seconds.
  pub max_age: u64,
  /// Number of generations to retain entries in the memory front cache.
  pub max_memory_generations: MaxMemoryGenerations,
}
