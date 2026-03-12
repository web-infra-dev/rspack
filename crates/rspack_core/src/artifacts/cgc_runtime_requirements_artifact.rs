use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

use crate::{ArtifactExt, ChunkUkey, RuntimeGlobals, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct CgcRuntimeRequirementsArtifact(FxHashMap<ChunkUkey, RuntimeGlobals>);

impl ArtifactExt for CgcRuntimeRequirementsArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS;
}

impl Deref for CgcRuntimeRequirementsArtifact {
  type Target = FxHashMap<ChunkUkey, RuntimeGlobals>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for CgcRuntimeRequirementsArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<FxHashMap<ChunkUkey, RuntimeGlobals>> for CgcRuntimeRequirementsArtifact {
  fn from(value: FxHashMap<ChunkUkey, RuntimeGlobals>) -> Self {
    Self(value)
  }
}

impl From<CgcRuntimeRequirementsArtifact> for FxHashMap<ChunkUkey, RuntimeGlobals> {
  fn from(value: CgcRuntimeRequirementsArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<FxHashMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item>
  for CgcRuntimeRequirementsArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <FxHashMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(FxHashMap::from_iter(iter))
  }
}

impl IntoIterator for CgcRuntimeRequirementsArtifact {
  type Item = <FxHashMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::Item;
  type IntoIter = <FxHashMap<ChunkUkey, RuntimeGlobals> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
