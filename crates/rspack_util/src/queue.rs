use std::{collections::VecDeque, hash::Hash};

use rustc_hash::FxHashSet as HashSet;

pub struct Queue<T: Hash + PartialEq + Eq + Clone> {
  q: VecDeque<T>,
  set: HashSet<T>,
}

impl<T: Hash + PartialEq + Eq + Clone> Default for Queue<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Hash + PartialEq + Eq + Clone> Queue<T> {
  pub fn new() -> Self {
    Self {
      q: VecDeque::default(),
      set: HashSet::default(),
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
