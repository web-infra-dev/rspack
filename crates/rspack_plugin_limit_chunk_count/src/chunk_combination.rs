use std::cmp::Ordering;
use std::hash::Hash;

use rspack_core::ChunkUkey;
use rspack_database::{Database, DatabaseItem, Ukey};

pub type ChunkCombinationUkey = Ukey<ChunkCombination>;

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

impl DatabaseItem for ChunkCombination {
  fn ukey(&self) -> Ukey<Self> {
    self.ukey
  }
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

pub struct ChunkCombinationBucket {
  combinations_by_ukey: Database<ChunkCombination>,
  sorted_combinations: Vec<ChunkCombinationUkey>,
  out_of_date: bool,
}

impl ChunkCombinationBucket {
  pub fn new() -> Self {
    Self {
      combinations_by_ukey: Default::default(),
      sorted_combinations: vec![],
      out_of_date: false,
    }
  }

  pub fn get_mut(&mut self, ukey: &ChunkCombinationUkey) -> &mut ChunkCombination {
    self.combinations_by_ukey.expect_get_mut(ukey)
  }

  pub fn add(&mut self, combination: ChunkCombination) {
    self.sorted_combinations.push(combination.ukey);
    self.combinations_by_ukey.add(combination);
    self.out_of_date = true;
  }

  fn sort_combinations(&mut self) {
    self.sorted_combinations.sort_by(|a_ukey, b_ukey| {
      let a = self.combinations_by_ukey.expect_get(a_ukey);
      let b = self.combinations_by_ukey.expect_get(b_ukey);
      // Layer 1: ordered by largest size benefit
      if a.size_diff < b.size_diff {
        Ordering::Less
      } else if a.size_diff > b.size_diff {
        Ordering::Greater
      } else {
        // Layer 2: ordered by smallest combined size
        if a.integrated_size < b.integrated_size {
          Ordering::Greater
        } else if a.integrated_size > b.integrated_size {
          Ordering::Less
        } else {
          // Layer 3: ordered by position difference in orderedChunk (-> to be deterministic)
          match a.b_idx.cmp(&b.a_idx) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => {
              // Layer 4: ordered by position in orderedChunk (-> to be deterministic)
              match a.b_idx.cmp(&b.b_idx) {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
              }
            }
          }
        }
      }
    });
    self.out_of_date = false;
  }

  pub fn pop_first(&mut self) -> Option<ChunkCombinationUkey> {
    if self.out_of_date {
      self.sort_combinations();
    }
    self.sorted_combinations.pop()
  }

  pub fn delete(&mut self, combination: &ChunkCombinationUkey) {
    self.out_of_date = true;
    self.sorted_combinations.retain(|ukey| ukey != combination);
  }

  pub fn update(&mut self) {
    self.out_of_date = true;
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

    let mut combinations = ChunkCombinationBucket::new();

    let combination_0 = ChunkCombinationUkey::new();
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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
    combinations.add(ChunkCombination {
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

    assert_eq!(combinations.pop_first().unwrap(), combination_0);

    combinations.delete(&combination_1);
    combinations.delete(&combination_3);
    combinations.delete(&combination_6);
    combinations.delete(&combination_10);

    let c = combinations.get_mut(&combination_2);
    c.a = chunk_1;
    c.integrated_size = 10074_f64;
    c.a_size = 10044_f64;
    c.size_diff = 10000_f64;
    combinations.update();

    let c = combinations.get_mut(&combination_4);
    c.a = chunk_1;
    c.integrated_size = 10074_f64;
    c.a_size = 10044_f64;
    c.size_diff = 10000_f64;
    combinations.update();

    let c = combinations.get_mut(&combination_7);
    c.a = chunk_1;
    c.integrated_size = 10066_f64;
    c.a_size = 10044_f64;
    c.size_diff = 10000_f64;
    combinations.update();

    let c = combinations.get_mut(&combination_8);
    c.a = chunk_1;
    c.integrated_size = 11450_f64;
    c.a_size = 10044_f64;
    c.size_diff = 9604_f64;
    combinations.update();

    assert_eq!(combinations.pop_first().unwrap(), combination_9);
  }
}
