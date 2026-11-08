use std::{
  collections::{BTreeMap, BTreeSet},
  sync::Arc,
};

use crate::{RspackHash, RspackHasher};

impl<T: RspackHash + ?Sized> RspackHash for &T {
  fn hash(&self, state: &mut RspackHasher) {
    (*self).hash(state);
  }
}

impl<T: RspackHash + ?Sized> RspackHash for Box<T> {
  fn hash(&self, state: &mut RspackHasher) {
    (**self).hash(state);
  }
}

impl<T: RspackHash + ?Sized> RspackHash for Arc<T> {
  fn hash(&self, state: &mut RspackHasher) {
    (**self).hash(state);
  }
}

impl<T: RspackHash> RspackHash for Option<T> {
  fn hash(&self, state: &mut RspackHasher) {
    if let Some(value) = self {
      value.hash(state);
    }
  }
}

fn hash_iter<T: RspackHash>(mut iter: impl Iterator<Item = T>, state: &mut RspackHasher) {
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

impl<T: RspackHash> RspackHash for [T] {
  fn hash(&self, state: &mut RspackHasher) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHash> RspackHash for Vec<T> {
  fn hash(&self, state: &mut RspackHasher) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHash + Ord> RspackHash for BTreeSet<T> {
  fn hash(&self, state: &mut RspackHasher) {
    hash_iter(self.iter(), state);
  }
}

impl<K: RspackHash + Ord, V: RspackHash> RspackHash for BTreeMap<K, V> {
  fn hash(&self, state: &mut RspackHasher) {
    hash_iter(self.iter(), state);
  }
}

impl<T: RspackHash, const N: usize> RspackHash for [T; N] {
  fn hash(&self, state: &mut RspackHasher) {
    hash_iter(self.iter(), state);
  }
}

impl<A: RspackHash, B: RspackHash> RspackHash for (A, B) {
  fn hash(&self, state: &mut RspackHasher) {
    state.write(b"(");
    self.0.hash(state);
    state.write(b",");
    self.1.hash(state);
    state.write(b")");
  }
}
