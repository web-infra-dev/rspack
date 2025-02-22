use std::{
  collections::{HashSet, VecDeque},
  hash::{BuildHasher, BuildHasherDefault, Hash},
};

use rspack_collections::{Identifier, IdentifierHasher};
use rustc_hash::FxBuildHasher;

pub struct Queue<T: Hash + PartialEq + Eq + Clone, S = FxBuildHasher> {
  q: VecDeque<T>,
  set: HashSet<T, S>,
}

impl<T: Hash + PartialEq + Eq + Clone> Default for Queue<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Hash + PartialEq + Eq + Clone, S: BuildHasher + Default> Queue<T, S> {
  pub fn new() -> Self {
    Self {
      q: VecDeque::default(),
      set: HashSet::<T, S>::default(),
    }
  }

  pub fn with_capacity(capacity: usize) -> Self {
    Self {
      q: VecDeque::with_capacity(capacity),
      set: HashSet::<T, S>::with_capacity_and_hasher(capacity, S::default()),
    }
  }

  pub fn enqueue(&mut self, item: T) {
    if !self.set.contains(&item) {
      self.q.push_back(item.clone());
      self.set.insert(item);
    }
  }

  pub fn dequeue(&mut self) -> Option<T> {
    if let Some(item) = self.q.pop_front() {
      self.set.remove(&item);
      return Some(item);
    }
    None
  }
}

pub type IdentifierQueue = Queue<Identifier, BuildHasherDefault<IdentifierHasher>>;
