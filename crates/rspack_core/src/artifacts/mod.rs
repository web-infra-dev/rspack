mod async_modules_artifact;
mod build_chunk_graph_artifact;
mod build_module_graph_artifact;
mod cgc_runtime_requirements_artifact;
mod cgm_hash_artifact;
mod cgm_runtime_requirement_artifact;
mod chunk_hashes_artifact;
mod chunk_ids_artifact;
mod chunk_render_artifact;
mod chunk_render_cache_artifact;
mod code_generate_cache_artifact;
mod code_generation_results;
mod dependencies_diagnostics_artifact;
mod imported_by_defer_modules_artifact;
mod module_graph_cache_artifact;
mod module_ids_artifact;
mod process_runtime_requirements_cache_artifact;
mod side_effects_do_optimize_artifact;

use std::{mem, sync::Arc};

use atomic_refcell::AtomicRefCell;

use crate::incremental::{Incremental, IncrementalPasses};

/// Trait for artifacts used in incremental compilation.
///
/// This trait associates each artifact with its corresponding incremental pass.
pub trait ArtifactExt: Sized {
  /// The incremental pass associated with this artifact.
  ///
  /// This constant defines which incremental compilation pass this artifact
  /// belongs to.
  const PASS: IncrementalPasses;

  /// Determines whether this artifact should be recovered from the previous compilation.
  ///
  /// Returns `true` if the artifact's pass is empty (always recover) or if
  /// the incremental system has readable mutations for this artifact's pass.
  fn should_recover(incremental: &Incremental) -> bool {
    incremental.mutations_readable(Self::PASS)
  }

  /// Recovers the artifact from the old compilation to the new compilation
  /// if the incremental pass allows it.
  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}

/// Recovers an artifact from the old compilation to the new compilation
/// if the incremental pass allows it.
pub fn recover_artifact<T: ArtifactExt>(incremental: &Incremental, new: &mut T, old: &mut T) {
  T::recover(incremental, new, old);
}

// Implementation for Box<T> - used when "napi" feature is disabled
// where BindingCell<T> is a type alias for Box<T>
#[cfg(not(feature = "napi"))]
impl<T: ArtifactExt> ArtifactExt for Box<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}

// Implementation for BindingCell<T> - used when "napi" feature is enabled
#[cfg(feature = "napi")]
impl<T: ArtifactExt + Into<crate::BindingCell<T>>> ArtifactExt for crate::BindingCell<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}

// Implementation for Arc<AtomicRefCell<T>> - used for shared artifacts
impl<T: ArtifactExt> ArtifactExt for Arc<AtomicRefCell<T>> {
  const PASS: IncrementalPasses = T::PASS;

  fn recover(incremental: &Incremental, new: &mut Self, old: &mut Self) {
    if Self::should_recover(incremental) {
      mem::swap(new, old);
    }
  }
}

pub use async_modules_artifact::AsyncModulesArtifact;
pub(crate) use build_chunk_graph_artifact::use_code_splitting_cache;
pub use build_chunk_graph_artifact::*;
pub use build_module_graph_artifact::*;
pub use cgc_runtime_requirements_artifact::CgcRuntimeRequirementsArtifact;
pub use cgm_hash_artifact::*;
pub use cgm_runtime_requirement_artifact::*;
pub use chunk_hashes_artifact::*;
pub use chunk_ids_artifact::*;
pub use chunk_render_artifact::ChunkRenderArtifact;
pub use chunk_render_cache_artifact::ChunkRenderCacheArtifact;
pub use code_generate_cache_artifact::CodeGenerateCacheArtifact;
pub use code_generation_results::*;
pub use dependencies_diagnostics_artifact::DependenciesDiagnosticsArtifact;
pub use imported_by_defer_modules_artifact::ImportedByDeferModulesArtifact;
pub use module_graph_cache_artifact::*;
pub use module_ids_artifact::ModuleIdsArtifact;
pub use process_runtime_requirements_cache_artifact::ProcessRuntimeRequirementsCacheArtifact;
pub use side_effects_do_optimize_artifact::*;
