use rspack_collections::UkeyMap;

use crate::{ArtifactExt, ChunkHashesResult, ChunkUkey, incremental::IncrementalPasses};

#[derive(Debug, Default)]
pub struct ChunkHashesArtifact {
  chunk_to_hashes: UkeyMap<ChunkUkey, ChunkHashesResult>,
}

impl ArtifactExt for ChunkHashesArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::CHUNKS_HASHES;
}

impl ChunkHashesArtifact {
  pub fn is_empty(&self) -> bool {
    self.chunk_to_hashes.is_empty()
  }

  pub fn get(&self, chunk: &ChunkUkey) -> Option<&ChunkHashesResult> {
    self.chunk_to_hashes.get(chunk)
  }

  pub fn set_hashes(&mut self, chunk: ChunkUkey, hashes: ChunkHashesResult) -> bool {
    if let Some(old) = self.chunk_to_hashes.get(&chunk)
      && old == &hashes
    {
      false
    } else {
      self.chunk_to_hashes.insert(chunk, hashes);
      true
    }
  }

  pub fn remove(&mut self, chunk: &ChunkUkey) -> Option<ChunkHashesResult> {
    self.chunk_to_hashes.remove(chunk)
  }

  pub fn retain(&mut self, f: impl FnMut(&ChunkUkey, &mut ChunkHashesResult) -> bool) {
    self.chunk_to_hashes.retain(f)
  }

  pub fn clear(&mut self) {
    self.chunk_to_hashes.clear();
  }
}
