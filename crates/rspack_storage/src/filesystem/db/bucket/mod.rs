mod index;
mod meta;
mod pack;

use futures::future::try_join_all;
use pack::{PackGenerator, PackId, PackIdAlloc};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use self::{meta::Meta, pack::Pack};
use super::ScopeFileSystem;
use crate::{Error, Result};

/// Bucket manages storage of key-value pairs using a pack-based strategy.
///
/// Data is organized into:
/// - Hot pack (ID 0): Frequently modified data, always loaded in memory
/// - Cold packs (ID 1+): Immutable data, loaded on-demand
///
/// Pack splitting occurs when hot pack exceeds max_pack_size.
#[derive(Debug)]
pub struct Bucket {
  meta: Meta,
  hot_pack: Pack,
  fs: ScopeFileSystem,
  max_pack_size: usize,
}

impl Bucket {
  /// Creates a new bucket, loading existing data or initializing empty.
  pub async fn new(fs: ScopeFileSystem, max_pack_size: usize) -> Result<Self> {
    fs.ensure_exist().await?;
    // Load or initialize metadata
    let meta = match Meta::load(&fs).await {
      Ok(meta) => meta,
      Err(e) if e.is_not_found() => Default::default(),
      Err(e) => return Err(e),
    };

    // Load hot pack and verify integrity
    let hot_pack = match Pack::load(&fs, PackIdAlloc::HOT_PACK_ID).await {
      Ok((pack, hash)) => {
        if !meta.hot_pack_index().check_content_hash(hash) {
          return Err(Error::CorruptedData(format!(
            "Hot pack '{}' content hash mismatch: expected {}, got {}",
            PackIdAlloc::HOT_PACK_ID.pack_name(),
            meta.hot_pack_index().content_hash(),
            hash
          )));
        }
        pack
      }
      Err(e) if e.is_not_found() => Default::default(),
      Err(e) => return Err(e),
    };

    Ok(Self {
      meta,
      hot_pack,
      fs,
      max_pack_size,
    })
  }

  /// Loads all key-value pairs from all packs (hot + cold).
  pub async fn load_all(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut result = self.hot_pack.clone().data();

    // Load and verify all cold packs
    let tasks = self
      .meta
      .cold_pack_indexes()
      .iter()
      .map(|(pack_id, index)| {
        let fs = self.fs.clone();
        let pack_id = *pack_id;
        async move {
          let (pack, hash) = Pack::load(&fs, pack_id).await?;
          if !index.check_content_hash(hash) {
            return Err(Error::CorruptedData(format!(
              "Pack '{}' content hash mismatch: expected {}, got {}",
              pack_id.pack_name(),
              index.content_hash(),
              hash
            )));
          }
          Ok(pack)
        }
      });
    let results = try_join_all(tasks).await?;
    for pack in results {
      result.extend(pack.data());
    }
    Ok(result)
  }

  /// Saves changes to disk, returning lists of added and removed files.
  ///
  /// Process:
  /// 1. Find packs affected by modified keys
  /// 2. Merge affected packs + hot pack + new data
  /// 3. Split into new hot pack and cold packs (by max_pack_size)
  /// 4. Write all packs and metadata
  ///
  /// # Arguments
  /// * `writable_fs` - Target filesystem (for transactions)
  /// * `data` - List of (key, value) pairs. None value = deletion
  ///
  /// # Returns
  /// (added_files, removed_files) for transaction commit
  pub async fn save(
    &mut self,
    writable_fs: Option<ScopeFileSystem>,
    data: Vec<(Vec<u8>, Option<Vec<u8>>)>,
  ) -> Result<(Vec<String>, Vec<String>)> {
    let writable_fs = writable_fs.unwrap_or(self.fs.clone());

    // Find packs that need to be rewritten (contain modified keys)
    let need_update_packs = self.need_update_packs(data.iter().map(|(k, _)| k)).await?;

    // Initialize pack generator for splitting
    let mut pack_generator = PackGenerator::new(self.max_pack_size);
    let mut removed_pack_ids: HashSet<_> = HashSet::default();
    for (pack_id, pack) in need_update_packs {
      removed_pack_ids.insert(pack_id);
      // Recycle pack id
      self.meta.update_pack_index(pack_id, None);
      pack_generator.extend(pack.data());
    }
    let hot_pack = std::mem::take(&mut self.hot_pack);
    pack_generator.extend(hot_pack.data());
    pack_generator.extend(
      data
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect(),
    );
    let (hot_pack, new_packs) = pack_generator.finish();

    // Alloc id for packs
    let mut pending_packs = Vec::with_capacity(new_packs.len() + 1);
    pending_packs.push((PackIdAlloc::HOT_PACK_ID, hot_pack));
    for pack in new_packs {
      let pack_id = self.meta.next_pack_id();
      pending_packs.push((pack_id, pack));
    }

    // Perform parallel writes
    let results = try_join_all(pending_packs.into_iter().map(|(pack_id, pack)| {
      let fs = writable_fs.clone();
      async move {
        let index = pack.save(&fs, pack_id).await?;
        Ok::<_, Error>((pack_id, pack, index))
      }
    }))
    .await?;

    // Update metadata with results
    let mut added_files = Vec::with_capacity(results.len());
    for (pack_id, pack, index) in results {
      if pack_id == PackIdAlloc::HOT_PACK_ID {
        self.hot_pack = pack;
      }
      self.meta.update_pack_index(pack_id, Some(index));
      // Remove reused pack id
      removed_pack_ids.remove(&pack_id);
      added_files.push(pack_id.pack_name());
    }

    // Collect removed pack files
    let removed_files = removed_pack_ids
      .into_iter()
      .map(|pack_id| pack_id.pack_name())
      .collect();

    // Write metadata
    self.meta.save(&writable_fs).await?;
    added_files.push(Meta::FILE_NAME.to_string());

    Ok((added_files, removed_files))
  }

  /// Finds and loads packs that contain any of the given keys.
  ///
  /// Returns only packs that had keys successfully removed (need rewriting).
  async fn need_update_packs(
    &mut self,
    keys: impl Iterator<Item = &Vec<u8>>,
  ) -> Result<HashMap<PackId, Pack>> {
    let mut packs = HashMap::default();
    let mut modified_pack_id = HashSet::default();

    for key in keys {
      // Check hot pack first (most common case)
      if self.meta.hot_pack_index().contains_key(key) {
        if self.hot_pack.remove(key) {
          continue; // Key removed from hot pack, no cold pack needed
        }
      }

      // Search cold packs using bloom filters
      for (pack_id, index) in self.meta.cold_pack_indexes() {
        if !index.contains_key(key) {
          continue; // Bloom filter says key not in this pack
        }

        // Load pack if not already loaded
        if !packs.contains_key(pack_id) {
          let (pack, hash) = Pack::load(&self.fs, *pack_id).await?;
          if !index.check_content_hash(hash) {
            return Err(Error::CorruptedData(format!(
              "Pack '{}' content hash mismatch: expected {}, got {}",
              pack_id.pack_name(),
              index.content_hash(),
              hash
            )));
          }
          packs.insert(*pack_id, pack);
        }

        // Try to remove key from pack
        let pack = packs.get_mut(pack_id).expect("pack must exist");
        if pack.remove(key) {
          modified_pack_id.insert(*pack_id);
          break; // Key found and removed
        }
      }
    }

    // Keep only packs that were actually modified
    packs.retain(|k, _| modified_pack_id.contains(k));
    Ok(packs)
  }
}

#[cfg(test)]
mod test {
  use itertools::Itertools;

  use super::{Bucket, Result, ScopeFileSystem};

  #[tokio::test]
  #[cfg_attr(miri, ignore)]
  async fn test_bucket() -> Result<()> {
    let fs = ScopeFileSystem::new_memory_fs("/bucket1".into());

    let mut bucket = Bucket::new(fs, 25).await?;
    assert_eq!(bucket.meta, Default::default());
    assert_eq!(bucket.hot_pack, Default::default());
    assert!(bucket.load_all().await?.is_empty());

    let data = (0..9)
      .into_iter()
      .map(|num| {
        (
          format!("key{num}").as_bytes().to_vec(),
          Some(format!("value{num}").as_bytes().to_vec()),
        )
      })
      .collect();
    bucket.save(None, data).await?;

    let data = bucket.load_all().await?;
    assert_eq!(data.len(), 9);
    for (i, (k, v)) in data.iter().sorted().enumerate() {
      assert_eq!(k, format!("key{i}").as_bytes());
      assert_eq!(v, format!("value{i}").as_bytes());
    }

    assert_eq!(bucket.meta.cold_pack_indexes().len(), 4);
    assert_eq!(bucket.hot_pack.clone().data().len(), 1);

    Ok(())
  }
}
