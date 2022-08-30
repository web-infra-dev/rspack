use crate::{Chunk, ChunkByUkey, ChunkUkey};

#[derive(Debug, Default)]
pub struct ChunkGraph {
  split_point_module_uri_to_chunk_ref: hashbrown::HashMap<String, ChunkUkey>,
}

impl ChunkGraph {
  pub fn add_chunk(&mut self, chunk: &Chunk) {
    self
      .split_point_module_uri_to_chunk_ref
      .insert(chunk.entry_uri.clone(), chunk.rid);
  }

  pub fn chunk_by_split_point_module_uri<'a>(
    &self,
    uri: &str,
    chunk_by_ukey: &'a ChunkByUkey,
  ) -> Option<&'a Chunk> {
    let ukey = self.split_point_module_uri_to_chunk_ref.get(uri)?;
    chunk_by_ukey.get(ukey)
  }
}
