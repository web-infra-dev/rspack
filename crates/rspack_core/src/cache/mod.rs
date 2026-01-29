mod disable;
mod memory;
mod mixed;
pub mod persistent;

use std::{fmt::Debug, sync::Arc};

use rspack_fs::{IntermediateFileSystem, ReadableFileSystem};

use self::{
  disable::DisableCache, memory::MemoryCache, mixed::MixedCache, persistent::PersistentCache,
};
use crate::{
  CacheOptions, Compilation, CompilerOptions,
  compilation::build_module_graph::BuildModuleGraphArtifact, incremental::Incremental,
};

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

  // BUILD_MODULE_GRAPH hooks
  async fn before_build_module_graph(
    &mut self,
    _make_artifact: &mut BuildModuleGraphArtifact,
    _incremental: &Incremental,
  ) {
  }
  async fn after_build_module_graph(&self, _make_artifact: &BuildModuleGraphArtifact) {}

  // FINISH_MODULES hooks
  async fn before_finish_modules(&mut self, _compilation: &mut Compilation) {}
  async fn after_finish_modules(&self, _compilation: &Compilation) {}

  // OPTIMIZE_DEPENDENCIES hooks
  async fn before_optimize_dependencies(&mut self, _compilation: &mut Compilation) {}
  async fn after_optimize_dependencies(&self, _compilation: &Compilation) {}

  // BUILD_CHUNK_GRAPH hooks
  async fn before_build_chunk_graph(&mut self, _compilation: &mut Compilation) {}
  async fn after_build_chunk_graph(&self, _compilation: &Compilation) {}

  // MODULE_IDS hooks
  async fn before_module_ids(&mut self, _compilation: &mut Compilation) {}
  async fn after_module_ids(&self, _compilation: &Compilation) {}

  // CHUNK_IDS hooks
  async fn before_chunk_ids(&mut self, _compilation: &mut Compilation) {}
  async fn after_chunk_ids(&self, _compilation: &Compilation) {}

  // MODULES_HASHES hooks
  async fn before_modules_hashes(&mut self, _compilation: &mut Compilation) {}
  async fn after_modules_hashes(&self, _compilation: &Compilation) {}

  // MODULES_CODEGEN hooks
  async fn before_modules_codegen(&mut self, _compilation: &mut Compilation) {}
  async fn after_modules_codegen(&self, _compilation: &Compilation) {}

  // MODULES_RUNTIME_REQUIREMENTS hooks
  async fn before_modules_runtime_requirements(&mut self, _compilation: &mut Compilation) {}
  async fn after_modules_runtime_requirements(&self, _compilation: &Compilation) {}

  // CHUNKS_RUNTIME_REQUIREMENTS hooks
  async fn before_chunks_runtime_requirements(&mut self, _compilation: &mut Compilation) {}
  async fn after_chunks_runtime_requirements(&self, _compilation: &Compilation) {}

  // CHUNKS_HASHES hooks
  async fn before_chunks_hashes(&mut self, _compilation: &mut Compilation) {}
  async fn after_chunks_hashes(&self, _compilation: &Compilation) {}

  // CHUNK_ASSET hooks
  async fn before_chunk_asset(&mut self, _compilation: &mut Compilation) {}
  async fn after_chunk_asset(&self, _compilation: &Compilation) {}

  // EMIT_ASSETS hooks
  async fn before_emit_assets(&mut self, _compilation: &mut Compilation) {}
  async fn after_emit_assets(&self, _compilation: &Compilation) {}

  /// Store old compilation for artifact recovery (used by MemoryCache)
  fn store_old_compilation(&mut self, _compilation: Box<Compilation>) {}
}

pub fn new_cache(
  compiler_path: &str,
  compiler_option: &Arc<CompilerOptions>,
  input_filesystem: Arc<dyn ReadableFileSystem>,
  intermediate_filesystem: Arc<dyn IntermediateFileSystem>,
) -> Box<dyn Cache> {
  match &compiler_option.cache {
    CacheOptions::Disabled => Box::new(DisableCache),
    CacheOptions::Memory { .. } => Box::<MemoryCache>::default(),
    CacheOptions::Persistent(option) => {
      let persistent = PersistentCache::new(
        compiler_path,
        option,
        compiler_option,
        input_filesystem,
        intermediate_filesystem,
      );
      Box::new(MixedCache::new(persistent))
    }
  }
}
