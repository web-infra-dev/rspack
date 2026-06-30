use std::{
  collections::{BTreeMap, BTreeSet},
  sync::Arc,
};

use crate::{RspackHash, RspackHashable};

impl<T: RspackHashable + ?Sized> RspackHashable for &T {
  fn hash(&self, state: &mut RspackHash) {
    (*self).hash(state);
  }
}

impl<T: RspackHashable + ?Sized> RspackHashable for Box<T> {
  fn hash(&self, state: &mut RspackHash) {
    (**self).hash(state);
  }
}

impl<T: RspackHashable + ?Sized> RspackHashable for Arc<T> {
  fn hash(&self, state: &mut RspackHash) {
    (**self).hash(state);
  }
}

impl<T: RspackHashable> RspackHashable for Option<T> {
  fn hash(&self, state: &mut RspackHash) {
    if let Some(value) = self {
      value.hash(state);
    }
  }
}

fn hash_iter<T: RspackHashable>(mut iter: impl Iterator<Item = T>, state: &mut RspackHash) {
  state.write(b"[");
  if let Some(item) = iter.next() {
    item.hash(state);
  }

  for item in iter {
    state.write(b",");
    item.hash(state);
  }
  state.write(b"]");
}

impl<T: RspackHashable> RspackHashable for [T] {
  fn hash(&self, state: &mut RspackHash) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHashable> RspackHashable for Vec<T> {
  fn hash(&self, state: &mut RspackHash) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHashable + Ord> RspackHashable for BTreeSet<T> {
  fn hash(&self, state: &mut RspackHash) {
    hash_iter(self.iter(), state);
  }
}

impl<K: RspackHashable + Ord, V: RspackHashable> RspackHashable for BTreeMap<K, V> {
  fn hash(&self, state: &mut RspackHash) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHashable, const N: usize> RspackHashable for [T; N] {
  fn hash(&self, state: &mut RspackHash) {
    hash_iter(self.iter(), state);
  }
}

impl<A: RspackHashable, B: RspackHashable> RspackHashable for (A, B) {
  fn hash(&self, state: &mut RspackHash) {
    state.write(b"(");
    self.0.hash(state);
    state.write(b",");
    self.1.hash(state);
    state.write(b")");
  }
}
