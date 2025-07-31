mod disable;
mod memory;
pub mod persistent;

use std::{fmt::Debug, sync::Arc};

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};

use self::{disable::DisableCache, memory::MemoryCache, persistent::PersistentCache};
use crate::{Compilation, CompilerOptions, ExperimentCacheOptions, make::MakeArtifact};

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
#[async_trait::async_trait]
pub trait Cache: Debug + Send + Sync {
  /// before compile return is_hot_start
  async fn before_compile(&mut self, _compilation: &mut Compilation) -> bool {
    false
  }
  async fn after_compile(&mut self, _compilation: &Compilation) {}

  async fn before_make(&mut self, _make_artifact: &mut MakeArtifact) {}
  async fn after_make(&mut self, _make_artifact: &MakeArtifact) {}
}

pub fn new_cache(
  compiler_path: &str,
  compiler_option: Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
) -> Box<dyn Cache> {
  match &compiler_option.experiments.cache {
    ExperimentCacheOptions::Disabled => Box::new(DisableCache),
    ExperimentCacheOptions::Memory => Box::new(MemoryCache),
    ExperimentCacheOptions::Persistent(option) => Box::new(PersistentCache::new(
      compiler_path,
      option,
      compiler_option.clone(),
      input_filesystem,
      intermediate_filesystem,
    )),
  }
}
