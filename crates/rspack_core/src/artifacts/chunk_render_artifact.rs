use std::ops::{Deref, DerefMut};

use rspack_collections::UkeyMap;

use crate::{ArtifactExt, ChunkRenderResult, ChunkUkey, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct ChunkRenderArtifact(UkeyMap<ChunkUkey, ChunkRenderResult>);

impl ArtifactExt for ChunkRenderArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNK_ASSET;
}

impl Deref for ChunkRenderArtifact {
  type Target = UkeyMap<ChunkUkey, ChunkRenderResult>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ChunkRenderArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<UkeyMap<ChunkUkey, ChunkRenderResult>> for ChunkRenderArtifact {
  fn from(value: UkeyMap<ChunkUkey, ChunkRenderResult>) -> Self {
    Self(value)
  }
}

impl From<ChunkRenderArtifact> for UkeyMap<ChunkUkey, ChunkRenderResult> {
  fn from(value: ChunkRenderArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item>
  for ChunkRenderArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(UkeyMap::from_iter(iter))
  }
}

impl IntoIterator for ChunkRenderArtifact {
  type Item = <UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item;
  type IntoIter = <UkeyMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
