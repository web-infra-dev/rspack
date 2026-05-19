use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use super::{BloomFilter, PackIndex};

/// Generator for building pack index during pack creation.
/// Accumulates bloom filter (for fast key lookup) and content hash (for integrity check).
#[derive(Default)]
pub struct IndexGenerator {
  bloom_filter: BloomFilter,
  content_hasher: FxHasher,
}

impl IndexGenerator {
  /// Add a key
  pub fn add_key(&mut self, key: &[u8]) {
    key.hash(&mut self.content_hasher);

    self.bloom_filter.insert(key);
  }

  /// Add a value
  pub fn add_value(&mut self, value: &[u8]) {
    value.hash(&mut self.content_hasher);
  }

  /// Consume the generator and produce the final PackIndex
  pub fn finish(self) -> PackIndex {
    PackIndex::new(self.bloom_filter, self.content_hasher.finish())
  }
}

#[cfg(test)]
mod test {
  use std::hash::{Hash, Hasher};

  use rustc_hash::FxHasher;

  use super::{BloomFilter, IndexGenerator};

  #[test]
  fn test_index_generator() {
    let mut generator = IndexGenerator::default();

    let data: Vec<(&[u8], &[u8])> = vec![
      ("key1".as_bytes(), "value1".as_bytes()),
      ("key2".as_bytes(), "value2".as_bytes()),
      ("key3".as_bytes(), "value3".as_bytes()),
    ];
    for (k, v) in &data {
      generator.add_key(k);
      generator.add_value(v);
    }
    let index = generator.finish();

    let mut content_hasher = FxHasher::default();
    for (k, v) in &data {
      k.hash(&mut content_hasher);
      v.hash(&mut content_hasher);
    }
    assert_eq!(index.content_hash, content_hasher.finish());

    let mut bloom_filter = BloomFilter::default();
    for (k, _) in &data {
      bloom_filter.insert(k);
    }
    assert_eq!(index.bloom_filter, bloom_filter);
  }
}
