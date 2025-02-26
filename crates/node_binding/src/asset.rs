use napi::{
  bindgen_prelude::{
    Array, Either, FromNapiValue, Object, ToNapiValue, TypeName, Unknown, ValidateNapiValue,
  },
  sys, NapiRaw,
};
use napi_derive::napi;
use rspack_napi::string::JsStringExt;
use rspack_napi_macros::field_names;
use rustc_hash::FxHashSet;

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
  pub related: Option<JsAssetInfoRelated>,
  /// unused css local ident for the css chunk
  pub css_unused_idents: Option<Vec<String>>,
  /// whether this asset is over the size limit
  pub is_over_size_limit: Option<bool>,
}

/// Webpack: AssetInfo = KnownAssetInfo & Record<string, any>
pub struct AssetInfo {
  known: KnownAssetInfo,
  extras: serde_json::Map<String, serde_json::Value>,
}

unsafe fn napi_value_to_json(
  env: sys::napi_env,
  value: Unknown,
) -> napi::Result<Option<serde_json::Value>> {
  match value.get_type()? {
    napi::ValueType::Null => Ok(Some(serde_json::Value::Null)),
    napi::ValueType::Boolean => {
      let b = value.coerce_to_bool()?.get_value()?;
      Ok(Some(serde_json::Value::Bool(b)))
    }
    napi::ValueType::Number => {
      let number = value.coerce_to_number()?.get_double()?;
      let f64_val = serde_json::Number::from_f64(number);
      match f64_val {
        Some(n) => Ok(Some(serde_json::Value::Number(n))),
        None => Ok(None),
      }
    }
    napi::ValueType::String => {
      let s = value.coerce_to_string()?.into_string();
      Ok(Some(serde_json::Value::String(s)))
    }
    napi::ValueType::Object => {
      let js_obj = value.coerce_to_object()?;
      let mut map = serde_json::Map::new();

      let names = Array::from_napi_value(env, js_obj.get_property_names()?.raw())?;
      for index in 0..names.len() {
        if let Some(name) = names.get::<String>(index)? {
          let prop_val = js_obj.get_named_property::<Unknown>(&name)?;
          if let Some(json_val) = napi_value_to_json(env, prop_val)? {
            map.insert(name, json_val);
          }
        }
      }

      Ok(Some(serde_json::Value::Object(map)))
    }
    napi::ValueType::Undefined
    | napi::ValueType::Symbol
    | napi::ValueType::Function
    | napi::ValueType::External
    | napi::ValueType::Unknown => Ok(None),
  }
}

impl FromNapiValue for AssetInfo {
  unsafe fn from_napi_value(env: sys::napi_env, napi_val: sys::napi_value) -> napi::Result<Self> {
    let known = KnownAssetInfo::from_napi_value(env, napi_val)?;
    let known_field_names = FxHashSet::from_iter(KnownAssetInfo::field_names());

    let mut extras = serde_json::Map::new();
    let js_obj = Object::from_napi_value(env, napi_val)?;
    let names = Array::from_napi_value(env, js_obj.get_property_names()?.raw())?;
    for index in 0..names.len() {
      if let Some(name) = names.get::<String>(index)? {
        if !known_field_names.contains(&name) {
          let value = js_obj.get_named_property::<Unknown>(&name)?;
          if let Some(json_value) = napi_value_to_json(env, value)? {
            extras.insert(name, json_value);
          }
        }
      }
    }

    Ok(Self { known, extras })
  }
}

impl ToNapiValue for AssetInfo {
  unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
    let napi_value = ToNapiValue::to_napi_value(env, val.known)?;
    let mut js_object = Object::from_napi_value(env, napi_value)?;
    for (key, value) in val.extras {
      js_object.set_named_property(&key, value)?;
    }
    Ok(napi_value)
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
      extras,
    }
  }
}

#[napi(object)]
pub struct JsAsset {
  pub name: String,
  pub info: AssetInfo,
}

impl From<rspack_core::AssetInfoRelated> for JsAssetInfoRelated {
  fn from(related: rspack_core::AssetInfoRelated) -> Self {
    Self {
      source_map: related.source_map,
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
