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

impl<'a> ChunkCombinationBucket {
  pub fn new() -> Self {
    Self {
      combinations_by_ukey: Default::default(),
      sorted_combinations: vec![],
      out_of_date: false,
    }
  }

  pub fn get_mut(&mut self, ukey: &ChunkCombinationUkey) -> Option<&mut ChunkCombination> {
    self.combinations_by_ukey.get_mut(ukey)
  }

  pub fn add(&mut self, combination: ChunkCombination) {
    self.sorted_combinations.push(combination.ukey);
    self.combinations_by_ukey.add(combination);
    self.out_of_date = true;
  }

  fn sort_combinations(&mut self) {
    self.sorted_combinations.sort_by(|a_ukey, b_ukey| {
      let a = self.combinations_by_ukey.get(a_ukey).unwrap();
      let b = self.combinations_by_ukey.get(b_ukey).unwrap();
      // Layer 1: ordered by largest size benefit
      if a.size_diff < b.size_diff {
        return Ordering::Greater;
      } else if a.size_diff > b.size_diff {
        return Ordering::Less;
      } else {
        // Layer 2: ordered by smallest combined size
        if a.integrated_size < b.integrated_size {
          return Ordering::Less;
        } else if a.integrated_size > b.integrated_size {
          return Ordering::Greater;
        } else {
          // Layer 3: ordered by position difference in orderedChunk (-> to be deterministic)
          if a.b_idx < b.a_idx {
            return Ordering::Less;
          } else if a.b_idx > b.a_idx {
            return Ordering::Greater;
          } else {
            // Layer 4: ordered by position in orderedChunk (-> to be deterministic)
            if a.b_idx < b.b_idx {
              return Ordering::Less;
            } else if a.b_idx > b.b_idx {
              return Ordering::Greater;
            } else {
              return Ordering::Equal;
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
