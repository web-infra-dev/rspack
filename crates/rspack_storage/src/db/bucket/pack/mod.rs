mod generator;
mod id;
mod id_alloc;

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

pub use self::{generator::PackGenerator, id::PackId, id_alloc::PackIdAlloc};
use super::{
  Error, Result,
  index::{IndexGenerator, PackIndex},
};
use crate::fs::ScopeFileSystem;

/// A pack file containing a collection of key-value pairs.
///
/// Pack files store data in a simple format:
/// - Each item has a header line: "key_len value_len"
/// - Followed by raw key bytes and value bytes
/// - Content hash is computed from all keys and values for integrity verification
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Pack {
  data: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Pack {
  pub fn new(data: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
    Self { data }
  }

  /// Loads a pack file from disk and returns the pack data with its content hash.
  ///
  /// Returns: (Pack, content_hash)
  pub async fn load(fs: &ScopeFileSystem, id: PackId) -> Result<(Self, u64)> {
    let pack_name = id.pack_name();
    let mut reader = fs.read_file(&pack_name).await?;

    let mut content_hasher = FxHasher::default();
    let mut data = vec![];
    while let Ok(header) = reader.read_line().await {
      if header.is_empty() {
        break;
      }
      let parts: Vec<_> = header.split(' ').collect();
      if parts.len() != 2 {
        return Err(Error::InvalidFormat(format!(
          "Invalid pack item header in '{}': expected 'key_len value_len', got '{}'",
          pack_name, header
        )));
      }

      let key_len = parts[0].parse::<usize>().map_err(|e| {
        Error::InvalidFormat(format!(
          "Failed to parse key length in '{}': invalid value '{}' ({})",
          pack_name, parts[0], e
        ))
      })?;
      let key = reader.read(key_len).await?;
      key.hash(&mut content_hasher);

      let value_len = parts[1].parse::<usize>().map_err(|e| {
        Error::InvalidFormat(format!(
          "Failed to parse value length in '{}': invalid value '{}' ({})",
          pack_name, parts[1], e
        ))
      })?;
      let value = reader.read(value_len).await?;
      value.hash(&mut content_hasher);

      data.push((key, value))
    }

    Ok((Self { data }, content_hasher.finish()))
  }

  /// Saves the pack to disk and generates its index metadata.
  ///
  /// The index includes a bloom filter for fast key lookups and a content hash for integrity.
  pub async fn save(&self, fs: &ScopeFileSystem, id: PackId) -> Result<PackIndex> {
    let mut writer = fs.write_file(id.pack_name()).await?;

    let mut index_gen = IndexGenerator::default();
    for (key, value) in &self.data {
      // header
      let header = format!("{} {}", key.len(), value.len());
      writer.write_line(&header).await?;

      // key
      writer.write(key).await?;
      index_gen.add_key(key);

      // value
      writer.write(value).await?;
      index_gen.add_value(value);
    }
    writer.flush().await?;
    Ok(index_gen.finish())
  }

  /// Consumes the pack and returns its underlying data.
  pub fn data(self) -> Vec<(Vec<u8>, Vec<u8>)> {
    self.data
  }

  /// Removes all items matching the given key from the pack.
  ///
  /// Returns `true` if at least one item was removed, `false` otherwise.
  pub fn remove(&mut self, key: &[u8]) -> bool {
    let original_len = self.data.len();
    self.data.retain(|(k, _)| k.as_slice() != key);
    // Check if length changed (more efficient than comparing with original_len != current_len)
    self.data.len() < original_len
  }
}

#[cfg(test)]
mod test {
  use super::{Pack, PackId, Result};
  use crate::fs::ScopeFileSystem;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_pack() -> Result<()> {
    let pack_id = PackId::new(10);
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());
    fs.ensure_exist().await?;

    // pack not found
    assert!(Pack::load(&fs, pack_id).await.is_err());

    let data: Vec<(Vec<u8>, Vec<u8>)> = vec![
      ("key1".into(), "value1".into()),
      ("key2".into(), "value2".into()),
      ("key3".into(), "value3".into()),
    ];
    let mut pack = Pack::new(data.clone());
    // check remove
    assert!(!pack.remove("key4".as_bytes()));
    assert!(pack.remove("key2".as_bytes()));

    let index = pack.save(&fs, pack_id).await?;
    let (other_pack, content_hash) = Pack::load(&fs, pack_id).await?;
    assert!(index.check_content_hash(content_hash));
    assert_eq!(pack, other_pack);

    // check index
    assert!(index.contains_key("key1".as_bytes()));
    // key2 has been removed
    assert!(!index.contains_key("key2".as_bytes()));
    assert!(index.contains_key("key3".as_bytes()));
    Ok(())
  }
}
