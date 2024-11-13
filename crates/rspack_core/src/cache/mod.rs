mod disable;
mod memory;
pub mod persistent;

use std::{fmt::Debug, sync::Arc};

use rspack_fs::ReadableFileSystem;

use self::{disable::DisableCache, memory::MemoryCache, persistent::PersistentCache};
use crate::{Compilation, CompilerOptions, ExperimentCacheOptions};

/// Cache trait
///
/// The cache trait provides a pair of methods that are called before and after the core build steps.
/// * before_<step_name>(): set or clean artifact to enable or disable incremental build
/// * after_<step_name>(): save artifact or nothing
///
/// ### Why not define it as a hook directly
/// * The design of cache is different from webpack.
/// * Hook is relatively complex.
/// * This API does not need to cooperate with the js side.
///
/// We can consider change to Hook when we need to open the API to js side.
pub trait Cache: Debug + Send + Sync {
  fn before_compile(&self, _compilation: &mut Compilation) {}
  fn after_compile(&self, _compilation: &Compilation) {}
}

pub fn new_cache(
  compiler_option: Arc<CompilerOptions>,
  fs: Arc<dyn ReadableFileSystem>,
) -> Arc<dyn Cache> {
  match &compiler_option.experiments.cache {
    ExperimentCacheOptions::Disabled => Arc::new(DisableCache),
    ExperimentCacheOptions::Memory => Arc::new(MemoryCache),
    ExperimentCacheOptions::Persistent(option) => Arc::new(PersistentCache::new(option, fs)),
  }
}
