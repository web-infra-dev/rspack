use std::fmt::Debug;

use napi::bindgen_prelude::*;

use super::WebpackSource;

#[napi(object)]
pub struct AssetContent {
  pub buffer: Option<Buffer>,
  pub source: Option<String>,
}
impl Debug for AssetContent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AssetContent")
      .field("buffer", &"buffer")
      .field("source", &self.source)
      .finish()
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct UpdateAssetOptions {
  pub asset: AssetContent,
  pub filename: String,
}

#[napi(object)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

#[napi(object)]
pub struct AssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  // pub immutable: bool,
  /// whether the asset is minimized
  pub minimized: bool,
  /// the value(s) of the full hash used for this asset
  // pub full_hash:
  /// the value(s) of the chunk hash used for this asset
  // pub chunk_hash:
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  // pub content_hash:
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  // pub source_filename:
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: bool,
  /// when asset ships data for updating an existing application (HMR)
  // pub hot_module_replacement:
  /// when asset is javascript and an ESM
  // pub javascript_module:
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
}

#[napi(object)]
pub struct Asset {
  pub name: String,
  pub source: WebpackSource,
  pub info: AssetInfo,
}

impl From<rspack_core::AssetInfoRelated> for AssetInfoRelated {
  fn from(related: rspack_core::AssetInfoRelated) -> Self {
    Self {
      source_map: related.source_map,
    }
  }
}

impl From<rspack_core::AssetInfo> for AssetInfo {
  fn from(info: rspack_core::AssetInfo) -> Self {
    Self {
      minimized: info.minimized,
      development: info.development,
      related: info.related.into(),
    }
  }
}
