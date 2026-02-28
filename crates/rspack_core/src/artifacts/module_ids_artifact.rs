use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierMap;

use crate::{ArtifactExt, ModuleId, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct ModuleIdsArtifact(IdentifierMap<ModuleId>);

impl ArtifactExt for ModuleIdsArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULE_IDS;
}

impl Deref for ModuleIdsArtifact {
  type Target = IdentifierMap<ModuleId>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ModuleIdsArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<IdentifierMap<ModuleId>> for ModuleIdsArtifact {
  fn from(value: IdentifierMap<ModuleId>) -> Self {
    Self(value)
  }
}

impl From<ModuleIdsArtifact> for IdentifierMap<ModuleId> {
  fn from(value: ModuleIdsArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<IdentifierMap<ModuleId> as IntoIterator>::Item> for ModuleIdsArtifact {
  fn from_iter<T: IntoIterator<Item = <IdentifierMap<ModuleId> as IntoIterator>::Item>>(
    iter: T,
  ) -> Self {
    Self(IdentifierMap::from_iter(iter))
  }
}

impl IntoIterator for ModuleIdsArtifact {
  type Item = <IdentifierMap<ModuleId> as IntoIterator>::Item;
  type IntoIter = <IdentifierMap<ModuleId> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
