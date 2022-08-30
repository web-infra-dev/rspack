use hashbrown::HashSet;

use crate::{ChunkByUkey, ChunkUkey};

#[derive(Debug, Default)]
pub struct ChunkGroup {
  pub(crate) chunks: Vec<ChunkUkey>,
}

impl ChunkGroup {
  pub fn get_files(&self, chunk_by_ukey: &ChunkByUkey) -> HashSet<String> {
    self
      .chunks
      .iter()
      .flat_map(|chunk_ukey| {
        chunk_by_ukey
          .get(chunk_ukey)
          .unwrap()
          .files
          .iter()
          .map(|file| file.to_string())
      })
      .collect()
  }
}
