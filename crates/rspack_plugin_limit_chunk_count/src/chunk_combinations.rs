use std::hash::Hash;

use rspack_core::ChunkUkey;

pub struct ChunkCombination {
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

pub struct ChunkCombinations {}

impl ChunkCombinations {
  pub fn new() -> Self {
    Self {}
  }

  pub fn add(&mut self, combination: ChunkCombination) {
    todo!();
  }

  pub fn pop_first(&mut self) -> Option<ChunkCombination> {
    todo!();
  }

  pub fn delete(&mut self, combination: &ChunkCombination) {
    todo!();
  }

  pub fn start_update(&mut self, combination: &ChunkCombination) -> Box<dyn FnMut(bool)> {
    todo!();
  }
}
