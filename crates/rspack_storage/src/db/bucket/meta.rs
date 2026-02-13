use rustc_hash::FxHashMap as HashMap;

use super::{
  Error, Result,
  index::PackIndex,
  pack::{PackId, PackIdAlloc},
};
use crate::fs::ScopeFileSystem;

/// Metadata for a bucket, tracking all pack files and their indexes.
///
/// Format:
/// ```text
/// pack_id_alloc (e.g., "1 2 5")
/// 0 content_hash bloom_filter  (hot pack)
/// 1 content_hash bloom_filter  (cold pack 1)
/// 2 content_hash bloom_filter  (cold pack 2)
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Meta {
  pack_id_alloc: PackIdAlloc,
  hot_pack_index: PackIndex,
  cold_pack_indexes: HashMap<PackId, PackIndex>,
}

impl Meta {
  pub const FILE_NAME: &str = "_meta";

  /// Parses a single index line: "pack_id content_hash bloom_filter"
  fn parse_index_line(line: &str) -> Option<(PackId, PackIndex)> {
    let (pack_id_str, index_str) = line.split_once(' ')?;
    let pack_id = pack_id_str.parse().ok()?;
    let index = index_str.parse().ok()?;
    Some((pack_id, index))
  }

  /// Loads metadata from file.
  pub async fn load(fs: &ScopeFileSystem) -> Result<Self> {
    let mut meta = Self::default();

    let mut reader = fs.read_file(&Self::FILE_NAME).await?;

    // First line: pack ID allocator state
    meta.pack_id_alloc = reader.read_line().await?.parse()?;

    // Remaining lines: pack indexes
    while let Ok(line) = reader.read_line().await {
      if line.is_empty() {
        break;
      }

      let Some((pack_id, pack_index)) = Self::parse_index_line(&line) else {
        return Err(Error::InvalidFormat(format!(
          "Failed to parse pack index in '{}': invalid line '{}'",
          Self::FILE_NAME,
          line
        )));
      };

      meta.update_pack_index(pack_id, Some(pack_index));
    }

    Ok(meta)
  }

  /// Saves metadata to file.
  pub async fn save(&self, fs: &ScopeFileSystem) -> Result<()> {
    let mut writer = fs.write_file(&Self::FILE_NAME).await?;

    // First line: pack ID allocator state
    writer.write_line(&self.pack_id_alloc.to_string()).await?;

    // Hot pack index (always ID 0)
    writer
      .write_line(&format!(
        "{} {}",
        PackIdAlloc::HOT_PACK_ID,
        self.hot_pack_index
      ))
      .await?;

    // Cold pack indexes
    for (pack_id, index) in self.cold_pack_indexes.iter() {
      writer.write_line(&format!("{pack_id} {index}")).await?;
    }

    writer.flush().await?;
    Ok(())
  }

  /// Allocates the next available pack ID.
  pub fn next_pack_id(&mut self) -> PackId {
    self.pack_id_alloc.next_id()
  }

  /// Updates or removes a pack index.
  ///
  /// - Some(index): Updates or inserts the index
  /// - None: Removes the pack (add ID to reuse pool for cold packs)
  pub fn update_pack_index(&mut self, id: PackId, index: Option<PackIndex>) {
    match index {
      Some(index) => {
        if id == PackIdAlloc::HOT_PACK_ID {
          self.hot_pack_index = index;
        } else {
          self.cold_pack_indexes.insert(id, index);
        }
      }
      None => {
        if id == PackIdAlloc::HOT_PACK_ID {
          unreachable!("Cannot remove hot pack index (ID 0)")
        } else {
          self.cold_pack_indexes.remove(&id);
          self.pack_id_alloc.add_id(id);
        }
      }
    }
  }

  /// Returns the hot pack index.
  pub fn hot_pack_index(&self) -> &PackIndex {
    &self.hot_pack_index
  }

  /// Returns all cold pack indexes.
  pub fn cold_pack_indexes(&self) -> &HashMap<PackId, PackIndex> {
    &self.cold_pack_indexes
  }
}

#[cfg(test)]
mod test {
  use super::{super::Pack, Meta, Result};
  use crate::fs::ScopeFileSystem;

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_meta() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());
    fs.ensure_exist().await?;

    // meta not found
    assert!(Meta::load(&fs).await.is_err());

    let pack = Pack::new(vec![("key1".into(), "value1".into())]);

    // new a meta
    let mut meta = Meta::default();
    let pack_id_1 = meta.pack_id_alloc.next_id();
    let index_1 = pack.save(&fs, pack_id_1).await?;
    meta.update_pack_index(pack_id_1, Some(index_1));

    let pack_id_2 = meta.pack_id_alloc.next_id();
    let index_2 = pack.save(&fs, pack_id_2).await?;
    meta.update_pack_index(pack_id_2, Some(index_2));

    let temp_id = meta.pack_id_alloc.next_id();
    meta.pack_id_alloc.add_id(temp_id);
    meta.save(&fs).await?;

    // test serialize and deserialize
    let other_meta = Meta::load(&fs).await?;
    assert_eq!(meta, other_meta);
    Ok(())
  }
}
