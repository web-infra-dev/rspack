use std::sync::Arc;

use dashmap::DashMap;
use futures::Future;
use rspack_core::CompilationAsset;
use rspack_error::{Error, Result};

use crate::MappedAsset;

#[derive(Debug, Clone)]
pub(crate) struct MappedAssetsCache(DashMap<Arc<str>, MappedAsset>);

impl MappedAssetsCache {
  pub(crate) fn new() -> Self {
    Self(DashMap::new())
  }

  pub(crate) async fn use_cache<'a, Assets, Handle, Return>(
    &self,
    assets: Assets,
    map_assets: Handle,
  ) -> Result<Vec<MappedAsset>>
  where
    Assets: Iterator<Item = (&'a String, &'a CompilationAsset)>,
    Handle: FnOnce(Vec<(String, &'a CompilationAsset)>) -> Return,
    Return: Future<Output = Result<Vec<MappedAsset>, Error>> + Send + 'a,
  {
    let capacity = assets.size_hint().1.unwrap_or_default();

    let mut mapped_asstes: Vec<MappedAsset> = Vec::with_capacity(capacity);
    let mut vanilla_assets = Vec::with_capacity(capacity);
    for (filename, vanilla_asset) in assets {
      if let Some((_, mapped_asset)) = self.0.remove(filename.as_str())
        && !vanilla_asset.info.version.is_empty()
        && vanilla_asset.info.version == mapped_asset.asset.1.info.version
      {
        mapped_asstes.push(mapped_asset);
        continue;
      }
      vanilla_assets.push((filename.to_owned(), vanilla_asset));
    }

    mapped_asstes.extend(map_assets(vanilla_assets).await?);

    self.0.clear();
    for mapped_asset in &mapped_asstes {
      let MappedAsset {
        asset: (filename, asset),
        ..
      } = mapped_asset;
      if !asset.info.version.is_empty() {
        self.0.insert(filename.clone(), mapped_asset.clone());
      }
    }

    Ok(mapped_asstes)
  }
}
