use crate::{Chunk, ChunkByRid, ChunkRid};

#[derive(Debug, Default)]
pub struct ChunkGraph {
  split_point_module_uri_to_chunk_ref: hashbrown::HashMap<String, ChunkRid>,
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
    chunk_by_rid: &'a ChunkByRid,
  ) -> Option<&'a Chunk> {
    let chunk_ref = self.split_point_module_uri_to_chunk_ref.get(uri)?;
    chunk_by_rid.get(chunk_ref)
  }
}
