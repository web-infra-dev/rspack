use super::{ChunkMap, ChunkSlotMap};
use crate::{Chunk, ChunkUkey};

/// Array-backed membership for current Chunks of one graph.
///
/// Stores the complete key, so removing a stale identity cannot erase the new
/// occupant of its slot. Historical identities need a HashSet instead.
#[derive(Debug, Default)]
pub struct ChunkSet {
  values: ChunkMap<()>,
}

impl ChunkSet {
  /// Reserve by slot high-water mark, including vacant slots, not live count.
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      values: ChunkMap::with_capacity(capacity),
    }
  }

  pub fn insert(&mut self, chunks: &ChunkSlotMap<Chunk>, key: ChunkUkey) -> bool {
    self.values.insert(chunks, key, ()).is_none()
  }

  pub fn remove(&mut self, key: &ChunkUkey) -> bool {
    self.values.remove(key).is_some()
  }

  pub fn contains(&self, key: &ChunkUkey) -> bool {
    self.values.contains(key)
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  pub fn iter(&self) -> impl Iterator<Item = &ChunkUkey> {
    self.values.keys()
  }

  pub fn retain(&mut self, mut keep: impl FnMut(&ChunkUkey) -> bool) {
    self.values.retain(|key, _| keep(key));
  }

  pub fn into_keys(self) -> impl Iterator<Item = ChunkUkey> {
    self.values.into_entries().map(|(key, ())| key)
  }
}
