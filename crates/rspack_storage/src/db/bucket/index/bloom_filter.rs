use super::{Error, Result};

const SEGMENT_SIZE: u64 = 64;
const SEGMENT_NUM: usize = 4;

/// A 256-bit Bloom filter implementation using 4 u64 values
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BloomFilter {
  bits: [u64; SEGMENT_NUM],
}

impl std::fmt::Display for BloomFilter {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let s = self
      .bits
      .iter()
      .map(|b| b.to_string())
      .collect::<Vec<_>>()
      .join(" ");
    write!(f, "{}", s)
  }
}

impl std::str::FromStr for BloomFilter {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.len() != SEGMENT_NUM {
      return Err(Error::InvalidFormat(format!(
        "Expected {} u64 segments for {}-bit bloom filter, got {} segments",
        SEGMENT_NUM,
        Self::BIT_SIZE,
        parts.len()
      )));
    }

    let mut bits = [0u64; SEGMENT_NUM];
    for (i, part) in parts.iter().enumerate() {
      let num = part.parse::<u64>().map_err(|e| {
        Error::InvalidFormat(format!(
          "Failed to parse bloom filter segment[{}] '{}': {}",
          i, part, e
        ))
      })?;
      bits[i] = num;
    }

    Ok(Self { bits })
  }
}

impl BloomFilter {
  const BIT_SIZE: u64 = SEGMENT_SIZE * (SEGMENT_NUM as u64);

  /// Calculate sum of bytes as a simple hash alternative (with wrapping on overflow)
  fn bytes_sum(bytes: &[u8]) -> u64 {
    bytes
      .iter()
      .fold(0u64, |acc, &b| acc.wrapping_add(b as u64))
  }

  /// Insert a key into the bloom filter
  pub fn insert(&mut self, key: &[u8]) {
    let sum = Self::bytes_sum(key);
    let pos = sum % Self::BIT_SIZE;
    let segment = (pos / SEGMENT_SIZE) as usize;
    let bit = (pos % SEGMENT_SIZE) as u32;
    self.bits[segment] |= 1u64 << bit;
  }

  /// Check if a key might be in the bloom filter
  pub fn contains(&self, key: &[u8]) -> bool {
    let sum = Self::bytes_sum(key);
    let pos = sum % Self::BIT_SIZE;
    let segment = (pos / SEGMENT_SIZE) as usize;
    let bit = (pos % SEGMENT_SIZE) as u32;
    (self.bits[segment] & (1u64 << bit)) != 0
  }
}

#[cfg(test)]
mod tests {
  use super::BloomFilter;

  #[test]
  fn test_bloom_filter() {
    let data = vec!["key1", "key2", "key3"];
    let mut filter = BloomFilter::default();
    // add
    for item in &data {
      filter.insert(item.as_bytes());
    }

    // check
    for item in &data {
      assert!(filter.contains(item.as_bytes()))
    }
    assert!(!filter.contains("key0".as_bytes()));

    // serialize and deserialize
    let serialized = filter.to_string();
    let deserialized: BloomFilter = serialized.parse().expect("should deserialize success");
    assert_eq!(deserialized, filter);
  }
}
