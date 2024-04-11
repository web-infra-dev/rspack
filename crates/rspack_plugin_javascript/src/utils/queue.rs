use std::{collections::VecDeque, hash::Hash};

use rustc_hash::FxHashSet as HashSet;

pub(crate) struct Queue<T> {
  q: VecDeque<T>,
  set: HashSet<T>,
}

impl<T: Hash + PartialEq + Eq + Copy + Clone> Queue<T> {
  pub(crate) fn new() -> Self {
    Self {
      q: VecDeque::default(),
      set: HashSet::default(),
    }
  }

  pub(crate) fn enqueue(&mut self, item: T) {
    if !self.set.contains(&item) {
      self.q.push_back(item);
      self.set.insert(item);
    }
  }

  pub(crate) fn dequeue(&mut self) -> Option<T> {
    if let Some(item) = self.q.pop_front() {
      self.set.remove(&item);
      return Some(item);
    }
    None
  }
}
