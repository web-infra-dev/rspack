use std::{collections::hash_set, slice};

use either::Either;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::ChunkUkey;

const INLINE_CAPACITY: usize = 4;

/// A module's chunk memberships, stored inline until there are more than four.
///
/// Cloning copies the memberships so chunk graph updates and module concatenation
/// can keep a snapshot while mutating the graph. Small snapshots avoid allocation.
/// Iteration order is unspecified, just like a hash set.
#[derive(Debug, Clone)]
pub struct ModuleChunks(Storage);

#[derive(Debug, Clone)]
enum Storage {
  Inline(SmallVec<[ChunkUkey; INLINE_CAPACITY]>),
  Heap(FxHashSet<ChunkUkey>),
}

impl Default for ModuleChunks {
  fn default() -> Self {
    Self(Storage::Inline(SmallVec::new()))
  }
}

impl ModuleChunks {
  #[inline]
  pub fn len(&self) -> usize {
    match &self.0 {
      Storage::Inline(chunks) => chunks.len(),
      Storage::Heap(chunks) => chunks.len(),
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }

  #[inline]
  pub fn contains(&self, chunk: &ChunkUkey) -> bool {
    match &self.0 {
      Storage::Inline(chunks) => chunks.contains(chunk),
      Storage::Heap(chunks) => chunks.contains(chunk),
    }
  }

  #[inline]
  pub fn insert(&mut self, chunk: ChunkUkey) -> bool {
    match &mut self.0 {
      Storage::Inline(chunks) => {
        if chunks.contains(&chunk) {
          return false;
        }
        if chunks.len() < INLINE_CAPACITY {
          chunks.push(chunk);
        } else {
          let mut set =
            FxHashSet::with_capacity_and_hasher(INLINE_CAPACITY + 1, Default::default());
          set.extend(chunks.iter().copied());
          set.insert(chunk);
          self.0 = Storage::Heap(set);
        }
        true
      }
      Storage::Heap(chunks) => chunks.insert(chunk),
    }
  }

  #[inline]
  pub fn remove(&mut self, chunk: &ChunkUkey) -> bool {
    match &mut self.0 {
      Storage::Inline(chunks) => {
        if let Some(index) = chunks.iter().position(|value| value == chunk) {
          chunks.swap_remove(index);
          true
        } else {
          false
        }
      }
      // Keep the allocation to avoid thrashing when memberships change repeatedly.
      Storage::Heap(chunks) => chunks.remove(chunk),
    }
  }

  pub fn clear(&mut self) {
    match &mut self.0 {
      Storage::Inline(chunks) => chunks.clear(),
      Storage::Heap(chunks) => chunks.clear(),
    }
  }

  #[inline]
  pub fn iter(&self) -> Either<slice::Iter<'_, ChunkUkey>, hash_set::Iter<'_, ChunkUkey>> {
    match &self.0 {
      Storage::Inline(chunks) => Either::Left(chunks.iter()),
      Storage::Heap(chunks) => Either::Right(chunks.iter()),
    }
  }

  pub fn is_subset(&self, other: &Self) -> bool {
    self.len() <= other.len() && self.iter().all(|chunk| other.contains(chunk))
  }
}

impl<'a> IntoIterator for &'a ModuleChunks {
  type Item = &'a ChunkUkey;
  type IntoIter = Either<slice::Iter<'a, ChunkUkey>, hash_set::Iter<'a, ChunkUkey>>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

impl IntoIterator for ModuleChunks {
  type Item = ChunkUkey;
  type IntoIter =
    Either<smallvec::IntoIter<[ChunkUkey; INLINE_CAPACITY]>, hash_set::IntoIter<ChunkUkey>>;

  fn into_iter(self) -> Self::IntoIter {
    match self.0 {
      Storage::Inline(chunks) => Either::Left(chunks.into_iter()),
      Storage::Heap(chunks) => Either::Right(chunks.into_iter()),
    }
  }
}
