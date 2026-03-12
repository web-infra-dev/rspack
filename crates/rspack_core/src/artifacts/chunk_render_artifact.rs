use std::ops::{Deref, DerefMut};

use rustc_hash::FxHashMap;

use crate::{ArtifactExt, ChunkRenderResult, ChunkUkey, incremental::IncrementalPasses};

#[derive(Debug, Default, Clone)]
pub struct ChunkRenderArtifact(FxHashMap<ChunkUkey, ChunkRenderResult>);

impl ArtifactExt for ChunkRenderArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNK_ASSET;
}

impl Deref for ChunkRenderArtifact {
  type Target = FxHashMap<ChunkUkey, ChunkRenderResult>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ChunkRenderArtifact {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl From<FxHashMap<ChunkUkey, ChunkRenderResult>> for ChunkRenderArtifact {
  fn from(value: FxHashMap<ChunkUkey, ChunkRenderResult>) -> Self {
    Self(value)
  }
}

impl From<ChunkRenderArtifact> for FxHashMap<ChunkUkey, ChunkRenderResult> {
  fn from(value: ChunkRenderArtifact) -> Self {
    value.0
  }
}

impl FromIterator<<FxHashMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item>
  for ChunkRenderArtifact
{
  fn from_iter<
    T: IntoIterator<Item = <FxHashMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item>,
  >(
    iter: T,
  ) -> Self {
    Self(FxHashMap::from_iter(iter))
  }
}

impl IntoIterator for ChunkRenderArtifact {
  type Item = <FxHashMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::Item;
  type IntoIter = <FxHashMap<ChunkUkey, ChunkRenderResult> as IntoIterator>::IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
