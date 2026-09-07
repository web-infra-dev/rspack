use crate::{
  BuildChunkGraphArtifact, Compilation, artifacts::ArtifactExt, incremental::IncrementalPasses,
  recover_artifact,
};

/// Owns the previous compilation artifacts used by incremental passes.
///
/// Artifact recovery is part of incremental compilation itself. It must stay
/// available regardless of which build cache implementation is selected.
#[derive(Debug, Default)]
pub(crate) struct IncrementalArtifacts {
  previous_compilation: Option<Box<Compilation>>,

  // The build chunk graph artifact is mutated by later passes, so recover it
  // from a snapshot captured immediately after its own pass instead of from
  // the final previous compilation.
  build_chunk_graph_artifact_snapshot: BuildChunkGraphArtifact,
}

impl IncrementalArtifacts {
  pub(crate) fn reset(&mut self) {
    self.previous_compilation = None;
    self.build_chunk_graph_artifact_snapshot = BuildChunkGraphArtifact::default();
  }

  pub(crate) fn store_previous_compilation(&mut self, compilation: Box<Compilation>) {
    self.previous_compilation = Some(compilation);
  }

  pub(crate) fn recover(&mut self, passes: IncrementalPasses, compilation: &mut Compilation) {
    if passes.contains(IncrementalPasses::BUILD_CHUNK_GRAPH)
      && BuildChunkGraphArtifact::should_recover(&compilation.incremental)
    {
      BuildChunkGraphArtifact::recover(
        &compilation.incremental,
        &mut compilation.build_chunk_graph_artifact,
        &mut self.build_chunk_graph_artifact_snapshot,
      );
    }

    let Some(previous) = self.previous_compilation.as_mut() else {
      return;
    };
    let incremental = &compilation.incremental;

    if passes.contains(IncrementalPasses::BUILD_MODULE_GRAPH) {
      recover_artifact(
        incremental,
        &mut compilation.build_module_graph_artifact,
        &mut previous.build_module_graph_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.exports_info_artifact,
        &mut previous.exports_info_artifact,
      );
    }

    if passes.contains(IncrementalPasses::FINISH_MODULES) {
      recover_artifact(
        incremental,
        &mut compilation.async_modules_artifact,
        &mut previous.async_modules_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.dependencies_diagnostics_artifact,
        &mut previous.dependencies_diagnostics_artifact,
      );
    }

    if passes.contains(IncrementalPasses::OPTIMIZE_DEPENDENCIES) {
      recover_artifact(
        incremental,
        &mut compilation.side_effects_optimize_artifact,
        &mut previous.side_effects_optimize_artifact,
      );
    }

    if passes.contains(IncrementalPasses::OPTIMIZE_CHUNK_MODULES) {
      recover_artifact(
        incremental,
        &mut compilation.imported_by_defer_modules_artifact,
        &mut previous.imported_by_defer_modules_artifact,
      );
    }

    if passes.contains(IncrementalPasses::MODULE_IDS) {
      recover_artifact(
        incremental,
        &mut compilation.module_ids_artifact,
        &mut previous.module_ids_artifact,
      );
    }

    if passes.contains(IncrementalPasses::CHUNK_IDS) {
      recover_artifact(
        incremental,
        &mut compilation.named_chunk_ids_artifact,
        &mut previous.named_chunk_ids_artifact,
      );
    }

    if passes.contains(IncrementalPasses::MODULES_HASHES) {
      recover_artifact(
        incremental,
        &mut compilation.cgm_hash_artifact,
        &mut previous.cgm_hash_artifact,
      );
    }

    if passes.contains(IncrementalPasses::MODULES_CODEGEN) {
      recover_artifact(
        incremental,
        &mut compilation.code_generation_results,
        &mut previous.code_generation_results,
      );
      recover_artifact(
        incremental,
        &mut compilation.code_generate_cache_artifact,
        &mut previous.code_generate_cache_artifact,
      );
    }

    if passes.contains(IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS) {
      recover_artifact(
        incremental,
        &mut compilation.cgm_runtime_requirements_artifact,
        &mut previous.cgm_runtime_requirements_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.process_runtime_requirements_cache_artifact,
        &mut previous.process_runtime_requirements_cache_artifact,
      );
    }

    if passes.contains(IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS) {
      recover_artifact(
        incremental,
        &mut compilation.cgc_runtime_requirements_artifact,
        &mut previous.cgc_runtime_requirements_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.runtime_proxy_metadata_artifact,
        &mut previous.runtime_proxy_metadata_artifact,
      );
    }

    if passes.contains(IncrementalPasses::CHUNKS_HASHES) {
      recover_artifact(
        incremental,
        &mut compilation.chunk_hashes_artifact,
        &mut previous.chunk_hashes_artifact,
      );
    }

    if passes.contains(IncrementalPasses::CHUNK_ASSET) {
      recover_artifact(
        incremental,
        &mut compilation.chunk_render_artifact,
        &mut previous.chunk_render_artifact,
      );
      recover_artifact(
        incremental,
        &mut compilation.chunk_render_cache_artifact,
        &mut previous.chunk_render_cache_artifact,
      );
    }
  }

  pub(crate) fn capture(&mut self, passes: IncrementalPasses, compilation: &mut Compilation) {
    if passes.contains(IncrementalPasses::BUILD_CHUNK_GRAPH)
      && BuildChunkGraphArtifact::should_recover(&compilation.incremental)
    {
      BuildChunkGraphArtifact::recover(
        &compilation.incremental,
        &mut self.build_chunk_graph_artifact_snapshot,
        &mut compilation.build_chunk_graph_artifact,
      );
    }
  }
}
