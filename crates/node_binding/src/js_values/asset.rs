use super::JsCompatSource;

#[napi(object)]
pub struct JsAssetInfoRelated {
  pub source_map: Option<String>,
}

impl From<JsAssetInfoRelated> for rspack_core::AssetInfoRelated {
  fn from(i: JsAssetInfoRelated) -> Self {
    Self {
      source_map: i.source_map,
    }
  }
}
#[napi(object)]
pub struct JsAssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  pub immutable: bool,
  /// whether the asset is minimized
  pub minimized: bool,
  /// the value(s) of the full hash used for this asset
  // pub full_hash:
  /// the value(s) of the chunk hash used for this asset
  // pub chunk_hash:
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  pub content_hash: Vec<String>,
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  // pub source_filename:
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: bool,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: bool,
  /// when asset is javascript and an ESM
  // pub javascript_module:
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: JsAssetInfoRelated,
}

impl From<JsAssetInfo> for rspack_core::AssetInfo {
  fn from(i: JsAssetInfo) -> Self {
    Self {
      immutable: i.immutable,
      minimized: i.minimized,
      development: i.development,
      hot_module_replacement: i.hot_module_replacement,
      related: i.related.into(),
      content_hash: i.content_hash.into_iter().collect(),
    }
  }
}

#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub source: Option<JsCompatSource>,
  pub info: JsAssetInfo,
}

impl From<rspack_core::AssetInfoRelated> for JsAssetInfoRelated {
  fn from(related: rspack_core::AssetInfoRelated) -> Self {
    Self {
      source_map: related.source_map,
    }
  }
}

impl From<rspack_core::AssetInfo> for JsAssetInfo {
  fn from(info: rspack_core::AssetInfo) -> Self {
    Self {
      immutable: info.immutable,
      minimized: info.minimized,
      development: info.development,
      hot_module_replacement: info.hot_module_replacement,
      related: info.related.into(),
      content_hash: info.content_hash.into_iter().collect(),
    }
  }
}
