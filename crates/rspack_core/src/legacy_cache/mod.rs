mod disable;
mod memory;
mod mixed;
pub mod persistent;

use std::{fmt::Debug, sync::Arc};

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};

use self::{
  disable::DisableCache, memory::MemoryCache, mixed::MixedCache, persistent::PersistentCache,
};
use crate::{CacheOptions, Compilation, CompilationLogging, CompilerOptions};

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
    CacheOptions::Disabled => Box::new(DisableCache),
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
