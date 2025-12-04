use rspack_collections::UkeyMap;

use crate::{ChunkUkey, chunk_graph_chunk::ChunkId};

#[derive(Debug, Default)]
pub struct ChunkNamedIdArtifact {
  pub chunk_short_names: UkeyMap<ChunkUkey, String>,
  pub chunk_long_names: UkeyMap<ChunkUkey, String>,
  pub chunk_ids: UkeyMap<ChunkUkey, ChunkId>,
}

impl ChunkNamedIdArtifact {
  pub fn clear(&mut self) {
    self.chunk_short_names.clear();
    self.chunk_long_names.clear();
    self.chunk_ids.clear();
  }

  pub fn retain<F>(&mut self, mut f: F)
  where
    F: FnMut(&ChunkUkey) -> bool,
  {
    self.chunk_short_names.retain(|chunk, _| f(chunk));
    self.chunk_long_names.retain(|chunk, _| f(chunk));
    self.chunk_ids.retain(|chunk, _| f(chunk));
  }

  pub fn remove(&mut self, chunk: &ChunkUkey) {
    self.chunk_short_names.remove(chunk);
    self.chunk_long_names.remove(chunk);
    self.chunk_ids.remove(chunk);
  }
}
