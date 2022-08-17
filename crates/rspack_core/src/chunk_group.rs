use hashbrown::HashSet;

use crate::{ChunkByRid, ChunkRid};

#[derive(Debug, Default)]
pub struct ChunkGroup {
  pub(crate) chunks: Vec<ChunkRid>,
}

impl ChunkGroup {
  pub fn get_files(&self, chunk_by_rid: &ChunkByRid) -> HashSet<String> {
    self
      .chunks
      .iter()
      .flat_map(|chunk_rid| {
        chunk_by_rid
          .get(chunk_rid)
          .unwrap()
          .files
          .iter()
          .map(|file| file.to_string())
      })
      .collect()
  }
}
