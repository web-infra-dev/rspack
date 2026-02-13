mod hot_pack;
mod meta;
mod pack;

use std::hash::{Hash, Hasher};

use rustc_hash::{FxHashMap as HashMap, FxHasher};

use self::{hot_pack::HotPack, meta::Meta, pack::Pack};
use super::error::{Error, Result};
use crate::fs::ScopeFileSystem;

/// Bucket manages pages for a scope
#[derive(Debug)]
pub struct Bucket {
  meta: Meta,
  hot_pack: HotPack,
  fs: ScopeFileSystem,
  max_pack_size: usize,
}

impl Bucket {
  pub async fn new(fs: ScopeFileSystem, max_pack_size: usize) -> Result<Self> {
    let meta = Meta::load(fs.clone()).await?;
    let (hot_pack, hash) = HotPack::load(fs.clone()).await?;
    let Some(index) = meta.indexes.get(&HotPack::id()) else {
      return Err(Error::InvalidFormat(
        "meta should contains hot pack info".into(),
      ));
    };
    if index.content_hash != hash {
      return Err(Error::CorruptedData("hot pack hash incorrect".into()));
    }
    Ok(Self {
      meta,
      hot_pack,
      fs,
      max_pack_size,
    })
  }

  pub async fn load_all(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
    let mut result = self.hot_pack.data.clone();
    for (pack_id, index) in &self.meta.indexes {
      if pack_id == &HotPack::id() {
        continue;
      }
      let (pack, hash) = Pack::load(self.fs.clone(), *pack_id).await?;
      if hash != index.content_hash {
        return Err(Error::CorruptedData(format!(
          "pack {pack_id} hash incorrect"
        )));
      }
      result.extend(pack.data);
    }
    Ok(result)
  }

  pub async fn save(
    &mut self,
    write_fs: Option<ScopeFileSystem>,
    data: Vec<(Vec<u8>, Option<Vec<u8>>)>,
  ) -> Result<(Vec<String>, Vec<String>)> {
    let write_fs = write_fs.unwrap_or(self.fs.clone());

    // TODO use vec to avoid hash
    let mut pack_info = HashMap::default();
    for (key, _) in &data {
      let mut hasher = FxHasher::default();
      key.hash(&mut hasher);
      let key_hash = hasher.finish();

      for (pack_id, index) in &self.meta.indexes {
        if index.bloom_filter & key_hash == 0 {
          continue;
        }

        // Ensure pack is loaded into pack_info
        if !pack_info.contains_key(pack_id) {
          let (pack, hash) = Pack::load(self.fs.clone(), *pack_id).await?;
          if hash != index.content_hash {
            return Err(Error::CorruptedData(format!(
              "pack {pack_id} hash incorrect"
            )));
          }
          pack_info.insert(*pack_id, pack);
        }

        // Now get mutable reference to the pack
        let pack = pack_info.get_mut(pack_id).unwrap();
        pack.data.retain(|item| &item.0 != key);
        break;
      }
    }

    let mut removed_files = Vec::with_capacity(pack_info.len());
    for pack_id in pack_info.keys() {
      removed_files.push(pack_id.pack_name());
      self.meta.indexes.remove(pack_id);
      self.meta.pack_id_alloc.add_id(*pack_id);
    }

    // TODO avoid generate large vec
    let pendding_data: Vec<_> = pack_info.into_values().flat_map(|pack| pack.data).collect();
    let packs = self.hot_pack.split(
      pendding_data,
      data
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v)))
        .collect(),
      self.max_pack_size,
    );
    let mut added_files = vec![];
    for item in packs {
      let pack_id = self.meta.pack_id_alloc.next_id();
      let index = item.save(write_fs.clone(), pack_id).await?;
      self.meta.indexes.insert(pack_id, index);
      added_files.push(pack_id.pack_name());
    }
    self.hot_pack.save(write_fs.clone()).await?;
    added_files.push(HotPack::id().pack_name());
    self.meta.save(write_fs.clone()).await?;
    added_files.push(Meta::FILE_NAME.to_string());

    Ok((added_files, removed_files))
  }
}
