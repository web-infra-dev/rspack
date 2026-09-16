use std::ops::{Deref, DerefMut};

use rayon::iter::ParallelIterator;

use super::ChunkSlotMap;
use crate::{Chunk, ChunkUkey};

/// Array-backed working data for the current Chunks of a single graph.
///
/// Uses the same storage as ChunkSlotMap, but cannot allocate Chunk identities.
/// Insertion validates membership against the owning graph's Chunk table before
/// replacing a slot. This prevents stale keys from evicting data for a reused
/// slot. Keep historical identities and sparse collections in HashMap instead.
#[derive(Debug)]
pub struct ChunkMap<T> {
  values: ChunkSlotMap<T>,
}

impl<T> Default for ChunkMap<T> {
  fn default() -> Self {
    Self::with_capacity(0)
  }
}

impl<T> ChunkMap<T> {
  /// Reserve using the owning table's slot_count, not its live Chunk count.
  /// During graph construction this can also include an estimate of new Chunks.
  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      values: ChunkSlotMap::with_capacity(capacity),
    }
  }

  /// Associate data with a current Chunk. Panics for a stale or foreign key.
  pub fn insert(&mut self, chunks: &ChunkSlotMap<Chunk>, key: ChunkUkey, value: T) -> Option<T> {
    assert!(
      chunks.contains(&key),
      "Cannot associate data with a non-member Chunk"
    );
    self.values.insert(key, value)
  }

  /// Remove only this exact identity. Also usable after the Chunk was deleted.
  pub fn remove(&mut self, key: &ChunkUkey) -> Option<T> {
    self.values.remove(key)
  }

  pub(super) fn retain(&mut self, keep: impl FnMut(&ChunkUkey, &mut T) -> bool) {
    self.values.retain(keep);
  }

  pub(crate) fn into_entries(self) -> impl Iterator<Item = (ChunkUkey, T)> {
    self.values.into_entries()
  }

  pub(crate) fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = (&ChunkUkey, &mut T)>
  where
    T: Send,
  {
    self.values.par_iter_mut()
  }

  pub fn get_or_insert_default(&mut self, chunks: &ChunkSlotMap<Chunk>, key: ChunkUkey) -> &mut T
  where
    T: Default,
  {
    assert!(
      chunks.contains(&key),
      "Cannot associate data with a non-member Chunk"
    );
    if !self.values.contains(&key) {
      self.values.insert(key, T::default());
    }
    self.values.expect_get_mut(&key)
  }
}

impl<T> Deref for ChunkMap<T> {
  type Target = ChunkSlotMap<T>;
  fn deref(&self) -> &Self::Target {
    &self.values
  }
}

impl<T> DerefMut for ChunkMap<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.values
  }
}
