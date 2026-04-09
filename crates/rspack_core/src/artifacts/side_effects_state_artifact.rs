use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierMap;

use crate::{ConnectionState, Module, ModuleIdentifier, OptimizationBailoutItem};

#[derive(Debug, Default, Clone)]
pub struct SideEffectsState {
  pub side_effect_free: bool,
  pub optimization_bailouts_to_add: Vec<OptimizationBailoutItem>,
  pub optimization_bailouts_to_remove: Vec<OptimizationBailoutItem>,
}

#[derive(Debug, Clone, Default)]
pub struct SideEffectsStateArtifact {
  states: IdentifierMap<SideEffectsState>,
  module_evaluation_states: IdentifierMap<ConnectionState>,
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
      states: value,
      module_evaluation_states: Default::default(),
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
      states: IdentifierMap::from_iter(iter),
      module_evaluation_states: Default::default(),
    }
  }
}

impl SideEffectsStateArtifact {
  /// Returns only the analyzed side-effects state overlay produced after parsing.
  /// This intentionally excludes `factory_meta.side_effect_free`.
  pub fn analyzed_side_effect_free(&self, module_identifier: &ModuleIdentifier) -> Option<bool> {
    self
      .get(module_identifier)
      .map(|state| state.side_effect_free)
  }

  pub fn module_evaluation_side_effects_state(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<ConnectionState> {
    self
      .module_evaluation_states
      .get(module_identifier)
      .copied()
  }

  pub fn set_module_evaluation_side_effects_states(
    &mut self,
    states: IdentifierMap<ConnectionState>,
  ) {
    self.module_evaluation_states = states;
  }
}

impl IntoIterator for SideEffectsStateArtifact {
  type Item = (ModuleIdentifier, SideEffectsState);
  type IntoIter = <IdentifierMap<SideEffectsState> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.states.into_iter()
  }
}

/// Returns the explicitly declared side-effects state from `package.json`, rules,
/// or other factory metadata. This must not be mixed with analyzed state when the
/// caller only wants user-declared semantics.
pub fn module_declared_side_effect_free(module: &dyn Module) -> Option<bool> {
  module
    .factory_meta()
    .and_then(|factory_meta| factory_meta.side_effect_free)
}

/// Returns the analyzed side-effects state derived during compilation.
/// This reads the deferred side-effects overlay first, then falls back to the
/// module's analyzed `build_meta.side_effect_free`.
pub fn module_analyzed_side_effect_free(
  module: &dyn Module,
  side_effects_state_artifact: &SideEffectsStateArtifact,
) -> Option<bool> {
  side_effects_state_artifact
    .analyzed_side_effect_free(&module.identifier())
    .or(module.build_meta().side_effect_free)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn module_evaluation_side_effects_states_round_trip() {
    let module_identifier = ModuleIdentifier::from("module-a");
    let mut artifact = SideEffectsStateArtifact::default();
    artifact.set_module_evaluation_side_effects_states(IdentifierMap::from_iter([(
      module_identifier,
      ConnectionState::Active(false),
    )]));

    assert_eq!(
      artifact.module_evaluation_side_effects_state(&module_identifier),
      Some(ConnectionState::Active(false))
    );
    assert_eq!(artifact.analyzed_side_effect_free(&module_identifier), None);
  }
}
