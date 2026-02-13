use super::Pack;

/// Pack generator that splits data into multiple packs based on size limits.
///
/// Strategy:
/// - Items > 80% of max_pack_size: Isolated in their own pack
/// - Regular items: Grouped together until max_pack_size is reached
/// - Remaining items: Become the hot pack
///
/// This ensures large items don't fragment packs and keeps pack sizes reasonable.
#[derive(Debug)]
pub struct PackGenerator {
  max_pack_size: usize,
  current_size: usize,
  data: Vec<(Vec<u8>, Vec<u8>)>,
  packs: Vec<Pack>,
}

impl PackGenerator {
  /// Creates a new pack generator with the specified maximum pack size.
  pub fn new(max_pack_size: usize) -> Self {
    Self {
      max_pack_size,
      current_size: 0,
      data: vec![],
      packs: vec![],
    }
  }

  /// Adds key-value pairs to the generator, automatically splitting into packs.
  ///
  /// Large items (>80% of max size) are isolated to avoid fragmentation.
  /// Regular items are grouped until a pack reaches max size.
  pub fn extend(&mut self, data: Vec<(Vec<u8>, Vec<u8>)>) {
    for (key, value) in data {
      let size = key.len() + value.len();

      // Isolate large items (>80% of max size) in their own pack
      if size > (self.max_pack_size * 4) / 5 {
        self.packs.push(Pack::new(vec![(key, value)]));
        continue;
      }

      // Check if adding this item would exceed max size
      if self.current_size + size > self.max_pack_size {
        // Flush current data into a new cold pack
        self.current_size = 0;
        let data = std::mem::take(&mut self.data);
        self.packs.push(Pack::new(data));
      }

      // Add item to current pack
      self.current_size += size;
      self.data.push((key, value));
    }
  }

  /// Finalizes the generation, returning (hot_pack, cold_packs).
  ///
  /// The hot pack contains remaining items and will be frequently modified.
  /// Cold packs are immutable and returned for writing to disk.
  pub fn finish(self) -> (Pack, Vec<Pack>) {
    (Pack::new(self.data), self.packs)
  }
}

#[cfg(test)]
mod test {
  use super::PackGenerator;
  #[test]
  fn test_pack_generator() {
    let mut generator = PackGenerator::new(25);
    generator.extend(
      (0..9)
        .into_iter()
        .map(|num| {
          (
            format!("key{num}").as_bytes().to_vec(),
            format!("value{num}").as_bytes().to_vec(),
          )
        })
        .collect(),
    );
    let (hot_pack, packs) = generator.finish();
    assert_eq!(packs.len(), 4);
    assert_eq!(hot_pack.data.len(), 1);
  }
}
