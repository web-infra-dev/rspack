use std::{
  ops::{Deref, DerefMut},
  sync::atomic::{AtomicU64, Ordering},
};

use rspack_collections::IdentifierMap;

use crate::{Module, ModuleIdentifier, OptimizationBailoutItem};

#[derive(Debug, Default, Clone)]
pub struct SideEffectsState {
  pub side_effect_free: bool,
  pub optimization_bailouts_to_add: Vec<OptimizationBailoutItem>,
  pub optimization_bailouts_to_remove: Vec<OptimizationBailoutItem>,
}

#[derive(Debug, Clone)]
pub struct SideEffectsStateArtifact {
  generation: u64,
  version: u64,
  states: IdentifierMap<SideEffectsState>,
}

static SIDE_EFFECTS_STATE_ARTIFACT_GENERATION: AtomicU64 = AtomicU64::new(1);

impl Default for SideEffectsStateArtifact {
  fn default() -> Self {
    Self {
      generation: SIDE_EFFECTS_STATE_ARTIFACT_GENERATION.fetch_add(1, Ordering::Relaxed),
      version: 0,
      states: IdentifierMap::default(),
    }
  }
}

impl Deref for SideEffectsStateArtifact {
  type Target = IdentifierMap<SideEffectsState>;

  fn deref(&self) -> &Self::Target {
    &self.states
  }
}

impl DerefMut for SideEffectsStateArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.states
  }
}

impl From<IdentifierMap<SideEffectsState>> for SideEffectsStateArtifact {
  fn from(value: IdentifierMap<SideEffectsState>) -> Self {
    Self {
      generation: SIDE_EFFECTS_STATE_ARTIFACT_GENERATION.fetch_add(1, Ordering::Relaxed),
      version: 0,
      states: value,
    }
  }
}

impl From<SideEffectsStateArtifact> for IdentifierMap<SideEffectsState> {
  fn from(value: SideEffectsStateArtifact) -> Self {
    value.states
  }
}

impl FromIterator<(ModuleIdentifier, SideEffectsState)> for SideEffectsStateArtifact {
  fn from_iter<T: IntoIterator<Item = (ModuleIdentifier, SideEffectsState)>>(iter: T) -> Self {
    Self {
      generation: SIDE_EFFECTS_STATE_ARTIFACT_GENERATION.fetch_add(1, Ordering::Relaxed),
      version: 0,
      states: IdentifierMap::from_iter(iter),
    }
  }
}

impl SideEffectsStateArtifact {
  pub fn generation(&self) -> u64 {
    self.generation
  }

  pub fn bump_version(&mut self) {
    self.version = self.version.wrapping_add(1);
  }

  pub fn version(&self) -> u64 {
    self.version
  }

  pub fn side_effect_free(&self, module_identifier: &ModuleIdentifier) -> Option<bool> {
    self
      .get(module_identifier)
      .map(|state| state.side_effect_free)
  }
}

impl IntoIterator for SideEffectsStateArtifact {
  type Item = (ModuleIdentifier, SideEffectsState);
  type IntoIter = <IdentifierMap<SideEffectsState> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.states.into_iter()
  }
}

pub fn module_side_effect_free(
  module: &dyn Module,
  side_effects_state_artifact: &SideEffectsStateArtifact,
) -> Option<bool> {
  module_declared_side_effect_free(module)
    .or_else(|| module_analyzed_side_effect_free(module, side_effects_state_artifact))
}

pub fn module_declared_side_effect_free(module: &dyn Module) -> Option<bool> {
  module
    .factory_meta()
    .and_then(|factory_meta| factory_meta.side_effect_free)
}

pub fn module_analyzed_side_effect_free(
  module: &dyn Module,
  side_effects_state_artifact: &SideEffectsStateArtifact,
) -> Option<bool> {
  side_effects_state_artifact
    .side_effect_free(&module.identifier())
    .or(module.build_meta().side_effect_free)
}
