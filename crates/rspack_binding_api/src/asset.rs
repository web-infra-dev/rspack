use napi::{
  Env, JsValue,
  bindgen_prelude::{
    Array, Either, FromNapiValue, JsObjectValue, Null, Object, ToNapiValue, TypeName, Unknown,
    ValidateNapiValue,
  },
  sys,
};
use napi_derive::napi;
use rspack_core::Reflector;
use rspack_napi::unknown_to_json_value;
use rspack_napi_macros::field_names;
use rustc_hash::FxHashSet;

#[derive(Clone)]
#[napi(object)]
pub struct AssetInfoRelated {
  pub source_map: Option<Either<String, Null>>,
}

impl From<AssetInfoRelated> for rspack_core::AssetInfoRelated {
  fn from(i: AssetInfoRelated) -> Self {
    Self {
      source_map: match i.source_map {
        Some(either) => match either {
          Either::A(string) => Some(string),
          Either::B(_) => None,
        },
        None => None,
      },
    }
  }
}

#[napi(object)]
#[field_names]
pub struct KnownAssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  pub immutable: Option<bool>,
  /// whether the asset is minimized
  pub minimized: Option<bool>,
  /// the value(s) of the full hash used for this asset
  pub fullhash: Option<Either<String, Vec<String>>>,
  /// the value(s) of the chunk hash used for this asset
  pub chunkhash: Option<Either<String, Vec<String>>>,
  /// the value(s) of the module hash used for this asset
  // pub modulehash:
  /// the value(s) of the content hash used for this asset
  pub contenthash: Option<Either<String, Vec<String>>>,
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  pub source_filename: Option<String>,
  /// when asset was created from a source file (potentially transformed), it should be flagged as copied
  pub copied: Option<bool>,
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: Option<bool>,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: Option<bool>,
  /// when asset is javascript and an ESM
  pub javascript_module: Option<bool>,
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: Option<AssetInfoRelated>,
  /// unused css local ident for the css chunk
  pub css_unused_idents: Option<Vec<String>>,
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,
  /// the asset type
  pub asset_type: Option<String>,
}

/// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
pub struct AssetInfo {
  known: KnownAssetInfo,
  extras: serde_json::Map<String, serde_json::Value>,
}

impl FromNapiValue for AssetInfo {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    unsafe {
      let known = KnownAssetInfo::from_napi_value(env, napi_val)?;
      let known_field_names = FxHashSet::from_iter(KnownAssetInfo::field_names());

      let mut extras = serde_json::Map::new();
      let object = Object::from_napi_value(env, napi_val)?;
      let names = Array::from_napi_value(env, object.get_property_names()?.raw())?;
      for index in 0..names.len() {
        if let Some(name) = names.get::<String>(index)?
          && !known_field_names.contains(&name)
        {
          let value = object.get_named_property::<Unknown>(&name)?;
          if let Some(json_value) = unknown_to_json_value(value)? {
            extras.insert(name, json_value);
          }
        }
      }

      Ok(Self { known, extras })
    }
  }
}

impl ToNapiValue for AssetInfo {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    unsafe {
      let napi_value = ToNapiValue::to_napi_value(env, val.known)?;
      let mut js_object = Object::from_napi_value(env, napi_value)?;
      for (key, value) in val.extras {
        js_object.set_named_property(&key, value)?;
      }
      Ok(napi_value)
    }
  }
}

impl TypeName for AssetInfo {
  fn type_name() -> &'static str {
    "AssetInfo"
  }
  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for AssetInfo {}

impl From<AssetInfo> for rspack_core::AssetInfo {
  fn from(js_value: AssetInfo) -> Self {
    let AssetInfo { known, extras } = js_value;

    let KnownAssetInfo {
      immutable,
      minimized,
      fullhash,
      chunkhash,
      contenthash,
      source_filename,
      copied,
      development,
      hot_module_replacement,
      javascript_module,
      related,
      css_unused_idents,
      is_over_size_limit,
      asset_type,
    } = known;

    let chunk_hash = chunkhash
      .map(|either| match either {
        Either::A(string) => FxHashSet::from_iter(vec![string]),
        Either::B(vec) => FxHashSet::from_iter(vec),
      })
      .unwrap_or_default();

    let full_hash = fullhash
      .map(|either| match either {
        Either::A(string) => FxHashSet::from_iter(vec![string]),
        Either::B(vec) => FxHashSet::from_iter(vec),
      })
      .unwrap_or_default();

    let content_hash = contenthash
      .map(|either| match either {
        Either::A(string) => FxHashSet::from_iter(vec![string]),
        Either::B(vec) => FxHashSet::from_iter(vec),
      })
      .unwrap_or_default();

    Self {
      immutable,
      minimized,
      chunk_hash,
      full_hash,
      content_hash,
      source_filename,
      copied,
      development,
      hot_module_replacement,
      javascript_module,
      related: related.map(Into::into).unwrap_or_default(),
      version: String::default(),
      css_unused_idents: css_unused_idents.map(|i| i.into_iter().collect()),
      is_over_size_limit,
      asset_type: asset_type.map(Into::into).unwrap_or_default(),
      extras,
    }
  }
}

impl AssetInfo {
  pub fn from_jsobject(env: &Env, object: &Object) -> napi::Result<Self> {
    // Safety: The Env and Object should be valid NAPI value
    unsafe { FromNapiValue::from_napi_value(env.raw(), object.raw()) }
  }

  pub fn get_related(&self) -> Option<AssetInfoRelated> {
    self.known.related.clone()
  }
}

#[napi(object, object_from_js = false)]
pub struct JsAsset {
  pub name: String,
  #[napi(ts_type = "AssetInfo")]
  pub info: Reflector,
}

impl From<rspack_core::AssetInfoRelated> for AssetInfoRelated {
  fn from(related: rspack_core::AssetInfoRelated) -> Self {
    Self {
      source_map: related.source_map.map(Either::A),
    }
  }
}

impl From<rspack_core::AssetInfo> for AssetInfo {
  fn from(value: rspack_core::AssetInfo) -> Self {
    let rspack_core::AssetInfo {
      immutable,
      minimized,
      full_hash,
      chunk_hash,
      content_hash,
      source_filename,
      copied,
      development,
      hot_module_replacement,
      javascript_module,
      related,
      css_unused_idents,
      is_over_size_limit,
      asset_type,
      extras,
      ..
    } = value;

    Self {
      known: KnownAssetInfo {
        immutable,
        minimized,
        development,
        hot_module_replacement,
        related: Some(related.into()),
        chunkhash: Some(Either::B(chunk_hash.into_iter().collect())),
        fullhash: Some(Either::B(full_hash.into_iter().collect())),
        contenthash: Some(Either::B(content_hash.into_iter().collect())),
        source_filename,
        copied,
        javascript_module,
        css_unused_idents: css_unused_idents.map(|i| i.into_iter().collect()),
        is_over_size_limit,
        asset_type: Some(asset_type.to_string()),
      },
      extras,
    }
  }
}

#[napi(object)]
pub struct JsAssetEmittedArgs {
  pub filename: String,
  pub output_path: String,
  pub target_path: String,
}
