use super::Cache;
use crate::{Compilation, recover_artifact};

/// Memory cache implementation
///
/// The memory cache stores the old compilation to recover artifacts
/// during incremental rebuilds.
#[derive(Debug, Default)]
pub struct MemoryCache {
  old_compilation: Option<Box<Compilation>>,
}

#[async_trait::async_trait]
impl Cache for MemoryCache {
  async fn before_compile(&mut self, _compilation: &mut Compilation) -> bool {
    self.old_compilation.is_some()
  }

  fn store_old_compilation(&mut self, compilation: Box<Compilation>) {
    self.old_compilation = Some(compilation);
  }

  // FINISH_MODULES: async_modules_artifact, dependencies_diagnostics_artifact
  async fn before_finish_modules(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.async_modules_artifact,
        &mut old_compilation.async_modules_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.dependencies_diagnostics_artifact,
        &mut old_compilation.dependencies_diagnostics_artifact,
      );
    }
  }

  // OPTIMIZE_DEPENDENCIES: side_effects_optimize_artifact
  async fn before_optimize_dependencies(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.side_effects_optimize_artifact,
        &mut old_compilation.side_effects_optimize_artifact,
      );
    }
  }

  // MODULE_IDS: module_ids_artifact
  async fn before_module_ids(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.module_ids_artifact,
        &mut old_compilation.module_ids_artifact,
      );
    }
  }

  // CHUNK_IDS: named_chunk_ids_artifact
  async fn before_chunk_ids(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.named_chunk_ids_artifact,
        &mut old_compilation.named_chunk_ids_artifact,
      );
    }
  }

  // MODULES_HASHES: cgm_hash_artifact
  async fn before_modules_hashes(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.cgm_hash_artifact,
        &mut old_compilation.cgm_hash_artifact,
      );
    }
  }

  // MODULES_CODEGEN: code_generation_results, code_generate_cache_artifact
  async fn before_modules_codegen(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.code_generation_results,
        &mut old_compilation.code_generation_results,
      );
      recover_artifact(
        incremental,
        &mut compilation.code_generate_cache_artifact,
        &mut old_compilation.code_generate_cache_artifact,
      );
    }
  }

  // MODULES_RUNTIME_REQUIREMENTS: cgm_runtime_requirements_artifact, process_runtime_requirements_cache_artifact
  async fn before_modules_runtime_requirements(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.cgm_runtime_requirements_artifact,
        &mut old_compilation.cgm_runtime_requirements_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.process_runtime_requirements_cache_artifact,
        &mut old_compilation.process_runtime_requirements_cache_artifact,
      );
    }
  }

  // CHUNKS_RUNTIME_REQUIREMENTS: cgc_runtime_requirements_artifact
  async fn before_chunks_runtime_requirements(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.cgc_runtime_requirements_artifact,
        &mut old_compilation.cgc_runtime_requirements_artifact,
      );
    }
  }

  // CHUNKS_HASHES: chunk_hashes_artifact
  async fn before_chunks_hashes(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.chunk_hashes_artifact,
        &mut old_compilation.chunk_hashes_artifact,
      );
    }
  }

  // CHUNK_ASSET: chunk_render_artifact, chunk_render_cache_artifact
  async fn before_chunk_asset(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.chunk_render_artifact,
        &mut old_compilation.chunk_render_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.chunk_render_cache_artifact,
        &mut old_compilation.chunk_render_cache_artifact,
      );
    }
  }
}
