use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

#[derive(Default)]
pub struct IndexGenerator {
  bloom_filter: u64,
  content_hasher: FxHasher,
}

impl IndexGenerator {
  pub fn hash_key(&mut self, key: &[u8]) {
    key.hash(&mut self.content_hasher);

    let mut key_hasher = FxHasher::default();
    key.hash(&mut key_hasher);
    self.bloom_filter = self.bloom_filter | key_hasher.finish();
  }
  pub fn hash_value(&mut self, value: &[u8]) {
    value.hash(&mut self.content_hasher);
  }
  pub fn finish(self) -> PackIndex {
    PackIndex {
      bloom_filter: self.bloom_filter,
      content_hash: self.content_hasher.finish(),
    }
  }
}

#[derive(Debug, Default)]
pub struct PackIndex {
  pub bloom_filter: u64,
  pub content_hash: u64,
}
