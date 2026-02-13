mod id;
mod id_alloc;
mod index;

use std::hash::{Hash, Hasher};

use rustc_hash::FxHasher;

use self::index::IndexGenerator;
pub use self::{id::PackId, id_alloc::PackIdAlloc, index::PackIndex};
use super::{Error, Result};
use crate::fs::ScopeFileSystem;

#[derive(Debug)]
pub struct Pack {
  pub data: Vec<(Vec<u8>, Vec<u8>)>,
}

impl Pack {
  pub fn new(data: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
    Self { data }
  }

  pub async fn load(fs: ScopeFileSystem, id: PackId) -> Result<(Self, u64)> {
    let mut reader = fs.read_file(id.pack_name()).await?;

    let mut content_hasher = FxHasher::default();
    let mut data = vec![];
    while let Ok(header) = reader.read_line().await {
      let header: Vec<_> = header.split(" ").collect();
      if header.len() != 2 {
        return Err(Error::InvalidFormat(format!(
          "parse pack item header failed, source: '{header:?}'"
        )));
      }

      let Ok(key_len) = header[0].parse::<usize>() else {
        return Err(Error::InvalidFormat(format!(
          "parse pack key length failed at '{}'",
          header[0]
        )));
      };
      let key = reader.read(key_len).await?;
      key.hash(&mut content_hasher);

      let Ok(value_len) = header[1].parse::<usize>() else {
        return Err(Error::InvalidFormat(format!(
          "parse pack value length failed at '{}'",
          header[1]
        )));
      };
      let value = reader.read(value_len).await?;
      value.hash(&mut content_hasher);

      data.push((key, value))
    }

    Ok((Self { data }, content_hasher.finish()))
  }

  /// Write Pack to bytes (consumes self)
  pub async fn save(&self, fs: ScopeFileSystem, id: PackId) -> Result<PackIndex> {
    let mut writer = fs.write_file(id.pack_name()).await?;

    let mut index_gen = IndexGenerator::default();
    for (key, value) in &self.data {
      // header
      let header = format!("{} {}", key.len(), value.len());
      writer.write_line(&header).await?;

      // key
      writer.write(key).await?;
      index_gen.hash_key(key);

      // value
      writer.write(value).await?;
      index_gen.hash_value(value);
    }
    writer.flush().await?;
    Ok(index_gen.finish())
  }
}
