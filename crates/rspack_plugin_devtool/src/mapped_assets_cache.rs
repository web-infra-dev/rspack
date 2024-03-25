use dashmap::DashMap;
use futures::Future;
use rspack_core::{CompilationAsset, CompilationAssets};
use rspack_error::{Error, Result};

use super::MappedAsset;

#[derive(Debug, Clone)]
pub struct MappedAssetsCache(DashMap<String, MappedAsset>);

impl MappedAssetsCache {
  pub fn new() -> Self {
    Self(DashMap::new())
  }

  pub async fn use_cache<'a, F, R>(
    &self,
    assets: &'a CompilationAssets,
    map_assets: F,
  ) -> Result<Vec<MappedAsset>>
  where
    F: FnOnce(Vec<(String, &'a CompilationAsset)>) -> R,
    R: Future<Output = Result<Vec<MappedAsset>, Error>> + Send + 'a,
  {
    let mut mapped_asstes: Vec<MappedAsset> = Vec::with_capacity(assets.len());
    let mut vanilla_assets = Vec::with_capacity(assets.len());
    for (filename, vanilla_asset) in assets {
      if let Some((_, MappedAsset { asset, source_map })) = self.0.remove(filename) {
        if !vanilla_asset.info.version.is_empty()
          && vanilla_asset.info.version == asset.1.info.version
        {
          mapped_asstes.push(MappedAsset { asset, source_map });
          continue;
        }
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
        self.0.insert(filename.to_owned(), mapped_asset.clone());
      }
    }

    Ok(mapped_asstes)
  }
}
