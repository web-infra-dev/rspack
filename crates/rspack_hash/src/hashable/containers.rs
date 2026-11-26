use std::{
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
  sync::Arc,
};

use crate::{RspackHash, RspackHasher};

/// Order-independent commutative accumulator for hashing unordered collections.
#[derive(Debug, Default)]
pub struct UnorderedHasher {
  entry_hasher: Option<RspackHasher>,
  len: usize,
  acc_sum: u64,
  acc_xor: u64,
}

impl UnorderedHasher {
  /// Creates an empty unordered collection hasher.
  pub fn new() -> Self {
    Self::default()
  }

  /// Returns whether no items or entries have been written yet.
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  fn entry_hasher(&mut self) -> &mut RspackHasher {
    match &mut self.entry_hasher {
      Some(hasher) => {
        hasher.reset_xxh3();
        hasher
      }
      slot @ None => slot.insert(RspackHasher::new_xxh3()),
    }
  }

  /// Hashes a single set item into the order-independent accumulator.
  pub fn write_item<T: RspackHash + ?Sized>(&mut self, item: &T) {
    let entry_hash = {
      let hasher = self.entry_hasher();
      item.hash(hasher);
      hasher.finish_xxh3_128()
    };
    self.mix(entry_hash);
  }

  /// Hashes a key-value entry into the order-independent accumulator.
  pub fn write_entry<K: RspackHash + ?Sized, V: RspackHash + ?Sized>(
    &mut self,
    key: &K,
    value: &V,
  ) {
    let entry_hash = {
      let hasher = self.entry_hasher();
      key.hash(hasher);
      let key_hash = hasher.finish_xxh3_128();
      hasher.reset_xxh3();
      hasher.write(&key_hash.to_le_bytes());
      value.hash(hasher);
      hasher.finish_xxh3_128()
    };
    self.mix(entry_hash);
  }

  fn mix(&mut self, entry_hash: u128) {
    let lo = entry_hash as u64;
    let hi = (entry_hash >> 64) as u64;
    self.len += 1;
    self.acc_sum = self.acc_sum.wrapping_add(lo);
    self.acc_xor ^= hi;
  }

  /// Writes the accumulated set digest into `state`.
  pub fn finish_set(&self, state: &mut RspackHasher) {
    self.finish(b"[]", b"[", b"]", state);
  }

  /// Writes the accumulated map digest into `state`.
  pub fn finish_map(&self, state: &mut RspackHasher) {
    self.finish(b"{}", b"{", b"}", state);
  }

  fn finish(&self, empty: &[u8], open: &[u8], close: &[u8], state: &mut RspackHasher) {
    if self.len == 0 {
      state.write(empty);
      return;
    }
    state.write(open);
    self.len.hash(state);
    state.write(b":");
    crate::write_u64_hex(self.acc_sum, state);
    state.write(b":");
    crate::write_u64_hex(self.acc_xor, state);
    state.write(close);
  }
}

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

impl<T: RspackHash, S> RspackHash for HashSet<T, S> {
  fn hash(&self, state: &mut RspackHasher) {
    let mut unordered = UnorderedHasher::new();
    for item in self {
      unordered.write_item(item);
    }
    unordered.finish_set(state);
  }
}

impl<K: RspackHash, V: RspackHash, S> RspackHash for HashMap<K, V, S> {
  fn hash(&self, state: &mut RspackHasher) {
    let mut unordered = UnorderedHasher::new();
    for (key, value) in self {
      unordered.write_entry(key, value);
    }
    unordered.finish_map(state);
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
