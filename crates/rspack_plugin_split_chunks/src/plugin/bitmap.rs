//! Dense chunk coordinates keep a set operation to a short contiguous word scan.
//! All bitmaps participating in one index have the same width.

#[derive(Clone, PartialEq, Eq, Hash)]
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

  pub(super) fn assign_intersection(&mut self, left: &Self, right: &Self) -> usize {
    debug_assert_eq!(self.words.len(), left.words.len());
    debug_assert_eq!(self.words.len(), right.words.len());
    let mut count = 0;
    for ((out, left), right) in self.words.iter_mut().zip(&left.words).zip(&right.words) {
      *out = left & right;
      count += out.count_ones() as usize;
    }
    count
  }

  // minChunks = 1 needs three predicates, not a population count for every
  // word. Both differences must be nonempty to exclude either original set.
  // A rejected intersection leaves scratch incomplete and must not be used.
  pub(super) fn assign_nonempty_proper_intersection<const EXCLUDE: bool>(
    &mut self,
    left: &Self,
    right: &Self,
    excluded: &Self,
  ) -> bool {
    debug_assert_eq!(self.words.len(), left.words.len());
    debug_assert_eq!(self.words.len(), right.words.len());
    debug_assert_eq!(self.words.len(), excluded.words.len());
    let (mut common, mut left_difference, mut right_difference) = (0, 0, 0);
    for (((out, left), right), excluded) in self
      .words
      .iter_mut()
      .zip(&left.words)
      .zip(&right.words)
      .zip(&excluded.words)
    {
      *out = left & right;
      if EXCLUDE && *out & excluded != 0 {
        return false;
      }
      common |= *out;
      left_difference |= left ^ *out;
      right_difference |= right ^ *out;
    }
    common != 0 && left_difference != 0 && right_difference != 0
  }

  // A rejected word proves that every supporting row together is too small.
  // On rejection the scratch contents are incomplete and must not be published.
  pub(super) fn assign_intersection_excluding(
    &mut self,
    left: &Self,
    right: &Self,
    excluded: &Self,
  ) -> Option<usize> {
    debug_assert_eq!(self.words.len(), left.words.len());
    debug_assert_eq!(self.words.len(), right.words.len());
    debug_assert_eq!(self.words.len(), excluded.words.len());
    let mut count = 0;
    for (((out, left), right), excluded) in self
      .words
      .iter_mut()
      .zip(&left.words)
      .zip(&right.words)
      .zip(&excluded.words)
    {
      *out = left & right;
      if *out & excluded != 0 {
        return None;
      }
      count += out.count_ones() as usize;
    }
    Some(count)
  }

  pub(super) fn assign(&mut self, other: &Self) {
    self.words.copy_from_slice(&other.words);
  }

  pub(super) fn equals_intersection(&self, left: &Self, right: &Self) -> bool {
    debug_assert_eq!(self.words.len(), left.words.len());
    debug_assert_eq!(self.words.len(), right.words.len());
    self
      .words
      .iter()
      .zip(&left.words)
      .zip(&right.words)
      .all(|((value, left), right)| *value == left & right)
  }

  pub(super) fn intersect_assign(&mut self, other: &Self) -> bool {
    debug_assert_eq!(self.words.len(), other.words.len());
    let mut remaining = 0;
    for (left, right) in self.words.iter_mut().zip(&other.words) {
      *left &= right;
      remaining |= *left;
    }
    remaining != 0
  }

  pub(super) fn ones(&self) -> impl Iterator<Item = usize> + '_ {
    self
      .words
      .iter()
      .copied()
      .enumerate()
      .flat_map(|(word, mut bits)| {
        std::iter::from_fn(move || {
          if bits == 0 {
            return None;
          }
          let bit = word * 64 + bits.trailing_zeros() as usize;
          bits &= bits - 1;
          Some(bit)
        })
      })
  }
}
