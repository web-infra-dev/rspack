use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierSet;

#[derive(Debug, Default, Clone)]
pub struct ImportedByDeferModulesArtifact(IdentifierSet);

impl Deref for ImportedByDeferModulesArtifact {
  type Target = IdentifierSet;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ImportedByDeferModulesArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<IdentifierSet> for ImportedByDeferModulesArtifact {
  fn from(value: IdentifierSet) -> Self {
    Self(value)
  }
}

impl From<ImportedByDeferModulesArtifact> for IdentifierSet {
  fn from(value: ImportedByDeferModulesArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<IdentifierSet as IntoIterator>::Item> for ImportedByDeferModulesArtifact {
  fn from_iter<T: IntoIterator<Item = <IdentifierSet as IntoIterator>::Item>>(iter: T) -> Self {
    Self(IdentifierSet::from_iter(iter))
  }
}

impl IntoIterator for ImportedByDeferModulesArtifact {
  type Item = <IdentifierSet as IntoIterator>::Item;
  type IntoIter = <IdentifierSet as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
