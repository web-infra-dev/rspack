use std::sync::Mutex;

use rspack_collections::{Identifier, IdentifierMap};

use crate::{ArtifactExt, ModuleId, incremental::IncrementalPasses};

/// Everything deterministic module ids depend on. If a rebuild produces equal
/// inputs, the previous ids are still correct. `candidates` uses the same sort
/// order as the assignment, so module graph iteration order doesn't matter.
#[derive(Debug, PartialEq, Eq)]
pub struct DeterministicModuleIdsInputs {
  pub candidates: Vec<(Identifier, String, Option<u32>)>,
  pub reserved_ids: Vec<ModuleId>,
  pub context: String,
  pub range: usize,
  pub max_length: usize,
  pub salt: usize,
  pub fixed_length: bool,
  pub fail_on_conflict: bool,
}

/// Kept between rebuilds to find modules whose id changed, and to let
/// deterministic ids skip reassignment.
#[derive(Debug, Default)]
pub struct ModuleIdsDiffArtifact {
  /// Module ids from when module hashes were last computed. New ids are compared
  /// against it. Cleared only after hashing succeeds, so a failed rebuild keeps
  /// it. Not set on cold builds, and never used to reserve ids.
  pub previous_ids: Option<IdentifierMap<ModuleId>>,
  /// Inputs of the last deterministic id assignment. A Mutex because the module
  /// ids hook only gets `&Compilation`.
  pub deterministic_inputs: Mutex<Option<DeterministicModuleIdsInputs>>,
}

impl ArtifactExt for ModuleIdsDiffArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULE_IDS;
}
