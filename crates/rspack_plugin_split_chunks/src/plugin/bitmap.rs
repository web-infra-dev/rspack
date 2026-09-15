//! Dense chunk coordinates keep a set operation to a short contiguous word scan.
//! All bitmaps participating in one index have the same width.

pub(super) struct ChunkBitmap {
  words: Box<[u64]>,
}

impl ChunkBitmap {
  pub(super) fn new(width: usize) -> Self {
    Self {
      words: vec![0; width.div_ceil(64)].into_boxed_slice(),
    }
  }

  pub(super) fn insert(&mut self, bit: usize) {
    self.words[bit / 64] |= 1u64 << (bit % 64);
  }

  pub(super) fn is_subset(&self, other: &Self) -> bool {
    debug_assert_eq!(self.words.len(), other.words.len());
    self
      .words
      .iter()
      .zip(&other.words)
      .all(|(left, right)| left & !right == 0)
  }
}
