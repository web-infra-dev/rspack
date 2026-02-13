use rustc_hash::FxHashMap as HashMap;

use super::{
  Error, Result,
  pack::{PackId, PackIdAlloc, PackIndex},
};
use crate::fs::ScopeFileSystem;

#[derive(Debug, Default)]
pub struct Meta {
  pub pack_id_alloc: PackIdAlloc,
  pub indexes: HashMap<PackId, PackIndex>,
}

impl Meta {
  pub const FILE_NAME: &str = "_meta";

  fn parse_line(line: &str) -> Option<(PackId, PackIndex)> {
    let options: Vec<_> = line.split(" ").collect();
    if options.len() != 3 {
      return None;
    }
    let Ok(pack_id) = options[0].try_into() else {
      return None;
    };
    let Ok(bloom_filter) = options[1].parse() else {
      return None;
    };
    let Ok(content_hash) = options[2].parse() else {
      return None;
    };

    Some((
      pack_id,
      PackIndex {
        bloom_filter,
        content_hash,
      },
    ))
  }

  /// Read Meta from file
  pub async fn load(fs: ScopeFileSystem) -> Result<Self> {
    let mut meta = Self::default();

    let mut reader = fs.read_file(&Self::FILE_NAME).await?;
    // first line
    meta.pack_id_alloc = PackIdAlloc::try_from_string(&reader.read_line().await?)?;

    while let Ok(line) = reader.read_line().await {
      let Some((pack_id, pack_index)) = Self::parse_line(&line) else {
        return Err(Error::InvalidFormat(format!(
          "parse bucket index failed at '{line}'"
        )));
      };
      meta.indexes.insert(pack_id, pack_index);
    }

    Ok(meta)
  }

  /// Write Meta to file
  pub async fn save(&self, fs: ScopeFileSystem) -> Result<()> {
    let mut writer = fs.write_file(&Self::FILE_NAME).await?;
    writer.write_line(&self.pack_id_alloc.to_string()).await?;

    for (pack_id, index) in self.indexes.iter() {
      writer
        .write_line(&format!(
          "{pack_id} {} {}",
          index.bloom_filter, index.content_hash
        ))
        .await?;
    }
    writer.flush().await?;
    Ok(())
  }
}
