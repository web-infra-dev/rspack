use std::ops::{Deref, DerefMut};

use rspack_collections::IdentifierSet;

use crate::{ArtifactExt, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct AsyncModulesArtifact(IdentifierSet);

impl ArtifactExt for AsyncModulesArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::INFER_ASYNC_MODULES;
}

impl Deref for AsyncModulesArtifact {
  type Target = IdentifierSet;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for AsyncModulesArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<IdentifierSet> for AsyncModulesArtifact {
  fn from(value: IdentifierSet) -> Self {
    Self(value)
  }
}

impl From<AsyncModulesArtifact> for IdentifierSet {
  fn from(value: AsyncModulesArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<IdentifierSet as IntoIterator>::Item> for AsyncModulesArtifact {
  fn from_iter<T: IntoIterator<Item = <IdentifierSet as IntoIterator>::Item>>(iter: T) -> Self {
    Self(IdentifierSet::from_iter(iter))
  }
}

impl IntoIterator for AsyncModulesArtifact {
  type Item = <IdentifierSet as IntoIterator>::Item;
  type IntoIter = <IdentifierSet as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
