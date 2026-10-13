use std::sync::Mutex;

use rspack_collections::{Identifier, IdentifierMap};

use crate::{ArtifactExt, ModuleId, incremental::IncrementalPasses};

/// Everything a deterministic module id assignment depends on. If a rebuild
/// produces the same inputs as the previous build, the previous ids are still
/// correct and the assignment can be skipped. `candidates` is sorted the same
/// way the assignment sorts modules, so module graph iteration order does not
/// affect the comparison.
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

/// State kept between rebuilds, so the module ids pass can tell which modules
/// got a different id, and deterministic ids can skip reassignment when nothing
/// they depend on has changed.
#[derive(Debug, Default)]
pub struct ModuleIdsDiffArtifact {
  /// Module ids as they were when module hashes were last computed. The module
  /// ids pass compares the new ids against this map. It is cleared only after
  /// hashing succeeds, so a rebuild that fails in between still compares
  /// against the ids the current hashes were built from. Cold builds leave it
  /// empty because they don't track changes. It is never used to reserve ids.
  pub previous_ids: Option<IdentifierMap<ModuleId>>,
  /// Inputs of the last deterministic id assignment, written by
  /// DeterministicModuleIdsPlugin. The module ids hook only gets `&Compilation`,
  /// hence the Mutex. The lock is held only to read or replace the value.
  pub deterministic_inputs: Mutex<Option<DeterministicModuleIdsInputs>>,
}

impl ArtifactExt for ModuleIdsDiffArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULE_IDS;
}
