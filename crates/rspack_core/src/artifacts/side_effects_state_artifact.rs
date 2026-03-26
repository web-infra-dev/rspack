use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierMap;

use crate::{ModuleIdentifier, OptimizationBailoutItem};

#[derive(Debug, Default, Clone)]
pub struct SideEffectsState {
  pub side_effect_free: bool,
  pub optimization_bailouts_to_add: Vec<OptimizationBailoutItem>,
  pub optimization_bailouts_to_remove: Vec<OptimizationBailoutItem>,
}

#[derive(Debug, Default, Clone)]
pub struct SideEffectsStateArtifact(IdentifierMap<SideEffectsState>);

impl Deref for SideEffectsStateArtifact {
  type Target = IdentifierMap<SideEffectsState>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for SideEffectsStateArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<IdentifierMap<SideEffectsState>> for SideEffectsStateArtifact {
  fn from(value: IdentifierMap<SideEffectsState>) -> Self {
    Self(value)
  }
}

impl From<SideEffectsStateArtifact> for IdentifierMap<SideEffectsState> {
  fn from(value: SideEffectsStateArtifact) -> Self {
    value.0
  }
}

impl FromIterator<(ModuleIdentifier, SideEffectsState)> for SideEffectsStateArtifact {
  fn from_iter<T: IntoIterator<Item = (ModuleIdentifier, SideEffectsState)>>(iter: T) -> Self {
    Self(IdentifierMap::from_iter(iter))
  }
}

impl IntoIterator for SideEffectsStateArtifact {
  type Item = (ModuleIdentifier, SideEffectsState);
  type IntoIter = <IdentifierMap<SideEffectsState> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
