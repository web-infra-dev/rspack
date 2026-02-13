mod bloom_filter;
mod generator;

pub use self::{bloom_filter::BloomFilter, generator::IndexGenerator};
use super::{Error, Result};

/// Index metadata for a pack file.
/// Contains a bloom filter for fast key existence checks and a content hash for integrity verification.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PackIndex {
  content_hash: u64,
  bloom_filter: BloomFilter,
}

impl std::fmt::Display for PackIndex {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.content_hash, self.bloom_filter)
  }
}

impl std::str::FromStr for PackIndex {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let Some((hash_str, bloom_str)) = s.split_once(' ') else {
      return Err(Error::InvalidFormat(format!(
        "Expected PackIndex format 'content_hash bloom_filter', got '{s}'"
      )));
    };
    let content_hash = hash_str.parse().map_err(|e| {
      Error::InvalidFormat(format!(
        "Failed to parse content_hash from '{hash_str}': {e}"
      ))
    })?;
    let bloom_filter = bloom_str.parse()?;

    Ok(Self {
      content_hash,
      bloom_filter,
    })
  }
}

impl PackIndex {
  pub fn new(bloom_filter: BloomFilter, content_hash: u64) -> Self {
    Self {
      bloom_filter,
      content_hash,
    }
  }

  /// Returns the content hash for this pack.
  pub fn content_hash(&self) -> u64 {
    self.content_hash
  }

  /// Check if a key might exist in the pack (bloom filter may have false positives)
  pub fn contains_key(&self, key: &[u8]) -> bool {
    self.bloom_filter.contains(key)
  }

  /// Verify pack content integrity by comparing content hash
  pub fn check_content_hash(&self, content_hash: u64) -> bool {
    self.content_hash == content_hash
  }
}

#[cfg(test)]
mod test {
  use std::hash::{Hash, Hasher};

  use rustc_hash::FxHasher;

  use super::{IndexGenerator, PackIndex};
  #[test]
  fn test_pack_index() {
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

    // check bloom filter
    for (k, _) in &data {
      assert!(index.contains_key(k));
    }
    assert!(!index.contains_key("some_other_key".as_bytes()));

    // check content hash
    let mut hasher = FxHasher::default();
    for (k, v) in &data {
      k.hash(&mut hasher);
      v.hash(&mut hasher);
    }
    assert!(index.check_content_hash(hasher.finish()));

    // content hash order sensitive
    let mut hasher = FxHasher::default();
    for (k, v) in data.iter().rev() {
      k.hash(&mut hasher);
      v.hash(&mut hasher);
    }
    assert!(!index.check_content_hash(hasher.finish()));

    // serialize and deserialize
    let index_str = index.to_string();
    let other_index: PackIndex = index_str.parse().expect("should parse success");
    assert_eq!(index, other_index);
  }
}
