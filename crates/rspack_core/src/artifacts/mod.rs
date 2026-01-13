use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use atomic_refcell::AtomicRefCell;
use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelIterator};
use rspack_collections::{IdentifierMap, IdentifierSet, UkeyMap};
use rspack_error::Diagnostic;

#[cfg(feature = "napi")]
use crate::BindingCell;
use crate::{
  ChunkRenderResult, ChunkUkey, DependencyId, DerefOption, ModuleId, ModuleIdentifier,
  RuntimeGlobals,
  incremental::{Incremental, IncrementalPasses},
};

/// Extension trait for artifacts that need to reset their state when passes are disabled.
/// Each artifact must declare its associated incremental pass via the `PASS` constant.
pub trait ArtifactExt {
  /// The incremental pass associated with this artifact.
  /// This ensures compile-time association between artifacts and their passes.
  const PASS: IncrementalPasses;

  /// Reset the artifact state.
  fn reset(&mut self);
}

// Blanket implementation for Box<T> (covers BindingCell type alias)
impl<T: ArtifactExt + ?Sized> ArtifactExt for Box<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn reset(&mut self) {
    (**self).reset();
  }
}

// Blanket implementation for Arc<AtomicRefCell<T>>
impl<T: ArtifactExt> ArtifactExt for Arc<AtomicRefCell<T>> {
  const PASS: IncrementalPasses = T::PASS;

  fn reset(&mut self) {
    self.borrow_mut().reset();
  }
}

// Blanket implementation for DerefOption<T>
impl<T: ArtifactExt + Default> ArtifactExt for DerefOption<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn reset(&mut self) {
    (**self).reset();
  }
}

// Blanket implementation for BindingCell<T> when napi feature is enabled
#[cfg(feature = "napi")]
impl<T: ArtifactExt> ArtifactExt for BindingCell<T> {
  const PASS: IncrementalPasses = T::PASS;

  fn reset(&mut self) {
    (**self).reset();
  }
}

/// Helper function to reset artifact state if the artifact's associated pass is disabled.
/// The pass is determined at compile-time from the artifact's `PASS` constant.
pub fn reset_artifact_if_passes_disabled<T: ArtifactExt>(
  incremental: &Incremental,
  artifact: &mut T,
) {
  if !incremental.passes_enabled(T::PASS) {
    artifact.reset();
  }
}

/// Macro to define a newtype artifact wrapper with associated pass.
macro_rules! define_artifact {
  (
    $(#[$meta:meta])*
    $name:ident($inner:ty) => $pass:ident
  ) => {
    $(#[$meta])*
    #[derive(Debug, Default, Clone)]
    pub struct $name($inner);

    impl $name {
      pub fn new(inner: $inner) -> Self {
        Self(inner)
      }
    }

    impl Deref for $name {
      type Target = $inner;

      fn deref(&self) -> &Self::Target {
        &self.0
      }
    }

    impl DerefMut for $name {
      fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
      }
    }

    impl ArtifactExt for $name {
      const PASS: IncrementalPasses = IncrementalPasses::$pass;

      fn reset(&mut self) {
        self.0.clear();
      }
    }
  };
}

// Define newtype artifacts with their associated passes
define_artifact!(
  /// Artifact for async modules inference
  AsyncModulesArtifact(IdentifierSet) => INFER_ASYNC_MODULES
);

define_artifact!(
  /// Artifact for modules imported by defer modules
  ImportedByDeferModulesArtifact(IdentifierSet) => INFER_ASYNC_MODULES
);

define_artifact!(
  /// Artifact for dependencies diagnostics collection
  DependenciesDiagnosticsArtifact(IdentifierMap<Vec<Diagnostic>>) => DEPENDENCIES_DIAGNOSTICS
);

define_artifact!(
  /// Artifact for module IDs assignment
  ModuleIdsArtifact(IdentifierMap<ModuleId>) => MODULE_IDS
);

define_artifact!(
  /// Artifact for chunk runtime requirements (CGC = Chunk Graph Chunk)
  CgcRuntimeRequirementsArtifact(UkeyMap<ChunkUkey, RuntimeGlobals>) => CHUNKS_RUNTIME_REQUIREMENTS
);

define_artifact!(
  /// Artifact for chunk render results
  ChunkRenderArtifact(UkeyMap<ChunkUkey, ChunkRenderResult>) => CHUNKS_RENDER
);

define_artifact!(
  /// Artifact for side effects optimization
  SideEffectsOptimizeArtifact(UkeyMap<crate::DependencyId, SideEffectsDoOptimize>) => SIDE_EFFECTS
);

// Additional trait implementations for DependenciesDiagnosticsArtifact
// These are needed for parallel iteration and collection in the codebase

impl FromParallelIterator<(ModuleIdentifier, Vec<Diagnostic>)> for DependenciesDiagnosticsArtifact {
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = (ModuleIdentifier, Vec<Diagnostic>)>,
  {
    Self(par_iter.into_par_iter().collect())
  }
}

impl Extend<(ModuleIdentifier, Vec<Diagnostic>)> for DependenciesDiagnosticsArtifact {
  fn extend<T: IntoIterator<Item = (ModuleIdentifier, Vec<Diagnostic>)>>(&mut self, iter: T) {
    self.0.extend(iter);
  }
}

impl IntoIterator for DependenciesDiagnosticsArtifact {
  type Item = (ModuleIdentifier, Vec<Diagnostic>);
  type IntoIter = <IdentifierMap<Vec<Diagnostic>> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl DependenciesDiagnosticsArtifact {
  /// Consumes the artifact and returns an iterator over the values
  pub fn into_values(self) -> impl Iterator<Item = Vec<Diagnostic>> {
    self.0.into_values()
  }
}

// Additional trait implementations for ChunkRenderArtifact
// These are needed for iteration and collection in the codebase

impl Extend<(ChunkUkey, ChunkRenderResult)> for ChunkRenderArtifact {
  fn extend<T: IntoIterator<Item = (ChunkUkey, ChunkRenderResult)>>(&mut self, iter: T) {
    self.0.extend(iter);
  }
}

impl IntoIterator for ChunkRenderArtifact {
  type Item = (ChunkUkey, ChunkRenderResult);
  type IntoIter = <UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

// Additional trait implementations for SideEffectsOptimizeArtifact
// These are needed for parallel iteration and collection in the codebase

impl FromParallelIterator<(DependencyId, SideEffectsDoOptimize)> for SideEffectsOptimizeArtifact {
  fn from_par_iter<I>(par_iter: I) -> Self
  where
    I: IntoParallelIterator<Item = (DependencyId, SideEffectsDoOptimize)>,
  {
    Self(par_iter.into_par_iter().collect())
  }
}

impl Extend<(DependencyId, SideEffectsDoOptimize)> for SideEffectsOptimizeArtifact {
  fn extend<T: IntoIterator<Item = (DependencyId, SideEffectsDoOptimize)>>(&mut self, iter: T) {
    self.0.extend(iter);
  }
}

impl IntoIterator for SideEffectsOptimizeArtifact {
  type Item = (DependencyId, SideEffectsDoOptimize);
  type IntoIter = <UkeyMap<DependencyId, SideEffectsDoOptimize> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

mod cgm_hash_artifact;
mod cgm_runtime_requirement_artifact;
mod chunk_hashes_artifact;
mod chunk_ids_artifact;
mod chunk_render_cache_artifact;
mod code_generation_results;
mod module_graph_cache_artifact;
mod module_static_cache_artifact;
mod side_effects_do_optimize_artifact;

pub use cgm_hash_artifact::*;
pub use cgm_runtime_requirement_artifact::*;
pub use chunk_hashes_artifact::*;
pub use chunk_ids_artifact::*;
pub use chunk_render_cache_artifact::ChunkRenderCacheArtifact;
pub use code_generation_results::*;
pub use module_graph_cache_artifact::*;
pub use module_static_cache_artifact::*;
pub use side_effects_do_optimize_artifact::*;
