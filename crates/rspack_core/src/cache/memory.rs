use super::Cache;
use crate::{BuildChunkGraphArtifact, Compilation, artifacts::ArtifactExt, recover_artifact};

/// Memory cache implementation
///
/// The memory cache stores the old compilation to recover artifacts
/// during incremental rebuilds.
#[derive(Debug, Default)]
pub struct MemoryCache {
  // this is used to recover from last compilation if the artifact is recoverable
  old_compilation: Option<Box<Compilation>>,

  // if the artifact itself is mutated | polluted in last compilation, we store the clone of the artifact here
  build_chunk_graph_artifact_snapshot: BuildChunkGraphArtifact,
}

#[async_trait::async_trait]
impl Cache for MemoryCache {
  async fn before_compile(&mut self, _compilation: &mut Compilation) -> bool {
    self.old_compilation.is_some()
  }

  fn store_old_compilation(&mut self, compilation: Box<Compilation>) {
    self.old_compilation = Some(compilation);
  }

  // BUILD_MODULE_GRAPH: build_module_graph_artifact (module graph recovery)
  async fn before_build_module_graph(&mut self, compilation: &mut Compilation) {
    if let Some(old_compilation) = self.old_compilation.as_mut() {
      let incremental = &compilation.incremental;
      recover_artifact(
        incremental,
        &mut compilation.build_module_graph_artifact,
        &mut old_compilation.build_module_graph_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.exports_info_artifact,
        &mut old_compilation.exports_info_artifact,
      );
    }
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

  // last build_chunk_graph_artifact is mutated and can't be used to recover, so we need to use clone snapshot to recover
  async fn before_build_chunk_graph(&mut self, compilation: &mut Compilation) {
    if BuildChunkGraphArtifact::should_recover(&compilation.incremental) {
      BuildChunkGraphArtifact::recover(
        &compilation.incremental,
        &mut compilation.build_chunk_graph_artifact,
        &mut self.build_chunk_graph_artifact_snapshot,
      );
    }
  }
  async fn after_build_chunk_graph(&mut self, compilation: &mut Compilation) {
    if BuildChunkGraphArtifact::should_recover(&compilation.incremental) {
      BuildChunkGraphArtifact::recover(
        &compilation.incremental,
        &mut self.build_chunk_graph_artifact_snapshot,
        &mut compilation.build_chunk_graph_artifact,
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

  // FIXME: migrate emitted_asset_versions to EmitAssetArtifact for recovery
  // EMIT_ASSETS: no artifacts to recover
  async fn before_emit_assets(&mut self, _compilation: &mut Compilation) {
    // No artifacts to recover for this phase
  }
}
