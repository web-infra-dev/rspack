use std::{cmp::Ordering, collections::BTreeSet, hash::Hash, sync::atomic::AtomicU32};

use rspack_core::ChunkUkey;
use rustc_hash::FxHashMap;

static NEXT_CHUNK_COMBINATION_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkCombinationUkey(u32, std::marker::PhantomData<ChunkCombination>);

impl ChunkCombinationUkey {
  pub fn new() -> Self {
    Self(
      NEXT_CHUNK_COMBINATION_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
      std::marker::PhantomData,
    )
  }
}

pub struct ChunkCombination {
  pub ukey: ChunkCombinationUkey,
  pub deleted: bool,
  pub size_diff: f64,
  pub a: ChunkUkey,
  pub b: ChunkUkey,
  pub integrated_size: f64,
  pub a_idx: usize,
  pub b_idx: usize,
  pub a_size: f64,
  pub b_size: f64,
}

impl PartialEq for ChunkCombination {
  fn eq(&self, other: &Self) -> bool {
    self.a == other.a && self.b == other.b
  }
}

impl Eq for ChunkCombination {}

impl Hash for ChunkCombination {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.a.hash(state);
    self.b.hash(state);
  }
}

/// The fields that order a combination, copied when it enters the bucket.
#[derive(Debug, Clone, Copy)]
struct SortKey {
  size_diff: f64,
  integrated_size: f64,
  a_idx: usize,
  b_idx: usize,
  ukey: ChunkCombinationUkey,
}

impl SortKey {
  fn new(combination: &ChunkCombination) -> Self {
    Self {
      size_diff: ordered_size_diff(combination.size_diff, combination.integrated_size),
      integrated_size: combination.integrated_size,
      a_idx: combination.a_idx,
      b_idx: combination.b_idx,
      ukey: combination.ukey,
    }
  }
}

/// `<` and `>` treat a NaN `size_diff` as equal to everything, so the order then falls
/// through to `integrated_size`. With finite options, `size_diff` is NaN only when
/// `integrated_size` is infinite. Ranking it at the matching end gives the same order
/// and keeps the order total for any option.
fn ordered_size_diff(size_diff: f64, integrated_size: f64) -> f64 {
  if !size_diff.is_nan() {
    size_diff
  } else if integrated_size == f64::NEG_INFINITY {
    f64::INFINITY
  } else {
    f64::NEG_INFINITY
  }
}

impl Ord for SortKey {
  /// The greatest key is merged first.
  fn cmp(&self, other: &Self) -> Ordering {
    // Layer 1: ordered by largest size benefit
    if self.size_diff < other.size_diff {
      return Ordering::Less;
    }
    if self.size_diff > other.size_diff {
      return Ordering::Greater;
    }
    // Layer 2: ordered by smallest combined size. A NaN size, which only non-finite
    // options produce, is merged last.
    match (
      self.integrated_size.is_nan(),
      other.integrated_size.is_nan(),
    ) {
      (false, false) => {
        if self.integrated_size < other.integrated_size {
          return Ordering::Greater;
        }
        if self.integrated_size > other.integrated_size {
          return Ordering::Less;
        }
      }
      (true, false) => return Ordering::Less,
      (false, true) => return Ordering::Greater,
      (true, true) => {}
    }
    // Layer 3: ordered by position difference in orderedChunk (-> to be deterministic)
    (other.b_idx - other.a_idx)
      .cmp(&(self.b_idx - self.a_idx))
      // Layer 4: ordered by position in orderedChunk (-> to be deterministic)
      .then_with(|| other.b_idx.cmp(&self.b_idx))
      // No two combinations share both positions, so this only keeps the order total.
      .then_with(|| self.ukey.cmp(&other.ukey))
  }
}

impl PartialOrd for SortKey {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl PartialEq for SortKey {
  fn eq(&self, other: &Self) -> bool {
    self.cmp(other) == Ordering::Equal
  }
}

impl Eq for SortKey {}

/// Live combinations, ordered so that `pop_first` returns the next one to merge.
pub struct ChunkCombinationBucket {
  combinations_by_ukey: FxHashMap<ChunkCombinationUkey, ChunkCombination>,
  sorted_combinations: BTreeSet<SortKey>,
}

impl ChunkCombinationBucket {
  pub fn new(combinations: Vec<ChunkCombination>) -> Self {
    let sorted_combinations = combinations.iter().map(SortKey::new).collect();
    let combinations_by_ukey = combinations
      .into_iter()
      .map(|combination| (combination.ukey, combination))
      .collect();
    Self {
      combinations_by_ukey,
      sorted_combinations,
    }
  }

  pub fn get(&self, ukey: &ChunkCombinationUkey) -> &ChunkCombination {
    self
      .combinations_by_ukey
      .get(ukey)
      .unwrap_or_else(|| panic!("ChunkCombination({ukey:?}) not found"))
  }

  fn get_mut(&mut self, ukey: &ChunkCombinationUkey) -> &mut ChunkCombination {
    self
      .combinations_by_ukey
      .get_mut(ukey)
      .unwrap_or_else(|| panic!("ChunkCombination({ukey:?}) not found"))
  }

  /// Removes the combination to merge next and marks it as deleted.
  pub fn pop_first(&mut self) -> Option<ChunkCombinationUkey> {
    let ukey = self.sorted_combinations.pop_last()?.ukey;
    self.get_mut(&ukey).deleted = true;
    Some(ukey)
  }

  /// Marks a live combination as deleted and removes it from the order.
  pub fn delete(&mut self, ukey: &ChunkCombinationUkey) {
    let combination = self
      .combinations_by_ukey
      .get_mut(ukey)
      .unwrap_or_else(|| panic!("ChunkCombination({ukey:?}) not found"));
    combination.deleted = true;
    let removed = self.sorted_combinations.remove(&SortKey::new(combination));
    debug_assert!(removed, "ChunkCombination({ukey:?}) is not in the order");
  }

  /// Changes a live combination and moves it to its new place in the order.
  pub fn update(
    &mut self,
    ukey: &ChunkCombinationUkey,
    update: impl FnOnce(&mut ChunkCombination),
  ) {
    let combination = self
      .combinations_by_ukey
      .get_mut(ukey)
      .unwrap_or_else(|| panic!("ChunkCombination({ukey:?}) not found"));
    let removed = self.sorted_combinations.remove(&SortKey::new(combination));
    debug_assert!(removed, "ChunkCombination({ukey:?}) is not in the order");
    update(combination);
    self.sorted_combinations.insert(SortKey::new(combination));
  }
}

#[cfg(test)]
mod test {
  use rspack_core::ChunkUkey;

  use super::*;

  #[test]
  fn pop_delete_and_update() {
    let chunk_0 = ChunkUkey::new();
    let chunk_1 = ChunkUkey::new();
    let chunk_2 = ChunkUkey::new();
    let chunk_3 = ChunkUkey::new();
    let chunk_4 = ChunkUkey::new();
    let chunk_5 = ChunkUkey::new();

    let mut list = vec![];

    let combination_0 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_0,
      a: chunk_0,
      a_idx: 0,
      a_size: 10022_f64,
      b: chunk_1,
      b_idx: 1,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_1 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_1,
      a: chunk_0,
      a_idx: 0,
      a_size: 10022_f64,
      b: chunk_2,
      b_idx: 2,
      b_size: 10030_f64,
      deleted: false,
      integrated_size: 10052_f64,
      size_diff: 10000_f64,
    });

    let combination_2 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_2,
      a: chunk_1,
      a_idx: 1,
      a_size: 10022_f64,
      b: chunk_2,
      b_idx: 2,
      b_size: 10030_f64,
      deleted: false,
      integrated_size: 10052_f64,
      size_diff: 10000_f64,
    });

    let combination_3 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_3,
      a: chunk_0,
      a_idx: 0,
      a_size: 10022_f64,
      b: chunk_3,
      b_idx: 3,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_4 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_4,
      a: chunk_1,
      a_idx: 1,
      a_size: 10022_f64,
      b: chunk_3,
      b_idx: 3,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_5 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_5,
      a: chunk_2,
      a_idx: 2,
      a_size: 10030_f64,
      b: chunk_3,
      b_idx: 3,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10052_f64,
      size_diff: 10000_f64,
    });

    let combination_6 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_6,
      a: chunk_0,
      a_idx: 0,
      a_size: 10022_f64,
      b: chunk_4,
      b_idx: 4,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_7 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_7,
      a: chunk_1,
      a_idx: 1,
      a_size: 10022_f64,
      b: chunk_4,
      b_idx: 4,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_8 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_8,
      a: chunk_2,
      a_idx: 2,
      a_size: 10030_f64,
      b: chunk_4,
      b_idx: 4,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10052_f64,
      size_diff: 10000_f64,
    });

    let combination_9 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_9,
      a: chunk_3,
      a_idx: 3,
      a_size: 10022_f64,
      b: chunk_4,
      b_idx: 4,
      b_size: 10022_f64,
      deleted: false,
      integrated_size: 10044_f64,
      size_diff: 10000_f64,
    });

    let combination_10 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_10,
      a: chunk_0,
      a_idx: 0,
      a_size: 10022_f64,
      b: chunk_5,
      b_idx: 5,
      b_size: 10010_f64,
      deleted: false,
      integrated_size: 11230_f64,
      size_diff: 9802_f64,
    });

    let combination_11 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_11,
      a: chunk_1,
      a_idx: 1,
      a_size: 10022_f64,
      b: chunk_5,
      b_idx: 5,
      b_size: 10010_f64,
      deleted: false,
      integrated_size: 11230_f64,
      size_diff: 9802_f64,
    });

    let combination_12 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_12,
      a: chunk_2,
      a_idx: 2,
      a_size: 10030_f64,
      b: chunk_5,
      b_idx: 5,
      b_size: 10010_f64,
      deleted: false,
      integrated_size: 11310_f64,
      size_diff: 9730_f64,
    });

    let combination_13 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_13,
      a: chunk_3,
      a_idx: 3,
      a_size: 10022_f64,
      b: chunk_5,
      b_idx: 5,
      b_size: 10010_f64,
      deleted: false,
      integrated_size: 11230_f64,
      size_diff: 9802_f64,
    });

    let combination_14 = ChunkCombinationUkey::new();
    list.push(ChunkCombination {
      ukey: combination_14,
      a: chunk_4,
      a_idx: 4,
      a_size: 10022_f64,
      b: chunk_5,
      b_idx: 5,
      b_size: 10010_f64,
      deleted: false,
      integrated_size: 11230_f64,
      size_diff: 9802_f64,
    });

    let mut combinations = ChunkCombinationBucket::new(list);

    assert_eq!(combinations.pop_first().unwrap(), combination_0);

    combinations.delete(&combination_1);
    combinations.delete(&combination_3);
    combinations.delete(&combination_6);
    combinations.delete(&combination_10);

    combinations.update(&combination_2, |c| {
      c.a = chunk_1;
      c.integrated_size = 10074_f64;
      c.a_size = 10044_f64;
      c.size_diff = 10000_f64;
    });

    combinations.update(&combination_4, |c| {
      c.a = chunk_1;
      c.integrated_size = 10074_f64;
      c.a_size = 10044_f64;
      c.size_diff = 10000_f64;
    });

    combinations.update(&combination_7, |c| {
      c.a = chunk_1;
      c.integrated_size = 10066_f64;
      c.a_size = 10044_f64;
      c.size_diff = 10000_f64;
    });

    combinations.update(&combination_8, |c| {
      c.a = chunk_1;
      c.integrated_size = 11450_f64;
      c.a_size = 10044_f64;
      c.size_diff = 9604_f64;
    });

    assert_eq!(combinations.pop_first().unwrap(), combination_9);
  }
}
