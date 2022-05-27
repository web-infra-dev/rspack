use crate::Chunk;

#[derive(Debug, Default)]
pub struct ChunkGraph {
  id_to_chunk: hashbrown::HashMap<String, Chunk>,
}

impl ChunkGraph {
  pub fn add_chunk(&mut self, chunk: Chunk) {
    self.id_to_chunk.insert(chunk.id.clone(), chunk);
  }

  pub fn chunk_by_id(&self, id: &str) -> Option<&Chunk> {
    self.id_to_chunk.get(id)
  }

  pub fn chunk_by_id_mut(&mut self, id: &str) -> Option<&mut Chunk> {
    self.id_to_chunk.get_mut(id)
  }

  pub fn remove_by_id(&mut self, id: &str) -> Option<Chunk> {
    self.id_to_chunk.remove(id)
  }

  pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
    self.id_to_chunk.values()
  }

  pub fn chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
    self.id_to_chunk.values_mut()
  }

  // FIXME: This is only used to render chunk in parallel, perhaps have a better to do it than just expose the raw data structure.
  pub fn id_to_chunk_mut(&mut self) -> &mut hashbrown::HashMap<String, Chunk> {
    &mut self.id_to_chunk
  }
}
