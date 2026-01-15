use std::ops::{Deref, DerefMut};

use rspack_collections::UkeyMap;

use crate::{ChunkUkey, RuntimeGlobals};

#[derive(Debug, Default, Clone)]
pub struct CgcRuntimeRequirementsArtifact(UkeyMap<ChunkUkey, RuntimeGlobals>);

impl Deref for CgcRuntimeRequirementsArtifact {
  type Target = UkeyMap<ChunkUkey, RuntimeGlobals>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for CgcRuntimeRequirementsArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<UkeyMap<ChunkUkey, RuntimeGlobals>> for CgcRuntimeRequirementsArtifact {
  fn from(value: UkeyMap<ChunkUkey, RuntimeGlobals>) -> Self {
    Self(value)
  }
}

impl From<CgcRuntimeRequirementsArtifact> for UkeyMap<ChunkUkey, RuntimeGlobals> {
  fn from(value: CgcRuntimeRequirementsArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<UkeyMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item>
  for CgcRuntimeRequirementsArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <UkeyMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(UkeyMap::from_iter(iter))
  }
}

impl IntoIterator for CgcRuntimeRequirementsArtifact {
  type Item = <UkeyMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item;
  type IntoIter = <UkeyMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
