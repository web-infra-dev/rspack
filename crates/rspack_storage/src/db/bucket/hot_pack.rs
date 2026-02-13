use super::{
  Result,
  pack::{Pack, PackId, PackIdAlloc, PackIndex},
};
use crate::fs::ScopeFileSystem;

#[derive(Debug)]
pub struct HotPack(Pack);

impl std::ops::Deref for HotPack {
  type Target = Pack;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl std::ops::DerefMut for HotPack {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl HotPack {
  pub fn id() -> PackId {
    PackIdAlloc::HOT_PACK_ID
  }

  pub async fn load(fs: ScopeFileSystem) -> Result<(Self, u64)> {
    let (pack, hash) = Pack::load(fs, Self::id()).await?;
    Ok((Self(pack), hash))
  }

  pub async fn save(&self, fs: ScopeFileSystem) -> Result<PackIndex> {
    self.0.save(fs, Self::id()).await
  }

  pub fn split(
    &mut self,
    cold: Vec<(Vec<u8>, Vec<u8>)>,
    hot: Vec<(Vec<u8>, Vec<u8>)>,
    max_pack_size: usize,
  ) -> Vec<Pack> {
    let data = std::mem::take(&mut self.data);
    let all = cold
      .into_iter()
      .chain(data.into_iter())
      .chain(hot.into_iter());
    let mut result = vec![];
    let mut current_pack_data = vec![];
    let mut current_pack_size = 0;
    for (key, value) in all {
      let size = key.len() + value.len();
      if size as f64 > 0.8_f64 * max_pack_size as f64 {
        // big item
        result.push(Pack::new(vec![(key, value)]));
        continue;
      }
      if current_pack_size + size > max_pack_size {
        // will over size
        let data = std::mem::take(&mut current_pack_data);
        current_pack_size = 0;
        result.push(Pack::new(data));
      }

      current_pack_size += size;
      current_pack_data.push((key, value));
    }
    self.data = current_pack_data;
    result
  }
}
