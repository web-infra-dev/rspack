use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::rspack_sources::SourceMap;

/// An owned map snapshot. Creating or returning the class does not serialize its map.
#[napi]
pub struct JsSourceMap {
  map: Box<SourceMap<'static>>,
}

#[napi]
impl JsSourceMap {
  #[napi(constructor)]
  pub fn new(json: String) -> napi::Result<Self> {
    SourceMap::from_json(json)
      .map(Self::from_map)
      .map_err(|error| napi::Error::from_reason(error.to_string()))
  }

  #[napi]
  pub fn to_json(&self) -> String {
    self.map.to_json()
  }
}

impl JsSourceMap {
  fn from_map(map: SourceMap<'static>) -> Self {
    Self {
      map: Box::new(map.into_shared()),
    }
  }
}

/// Convert a JS class into an owned, Send snapshot on the JS thread. The class
/// keeps its snapshot alive for loaders that retain their input map after returning.
pub struct JsSourceMapObject(JsSourceMap);

impl From<SourceMap<'static>> for JsSourceMapObject {
  fn from(map: SourceMap<'static>) -> Self {
    Self(JsSourceMap::from_map(map))
  }
}

impl JsSourceMapObject {
  pub fn into_map(self) -> SourceMap<'static> {
    *self.0.map
  }
}

impl FromNapiValue for JsSourceMapObject {
  unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
    let instance = unsafe { ClassInstance::<JsSourceMap>::from_napi_value(env, value)? };
    Ok(Self(JsSourceMap {
      map: Box::new(instance.map.as_ref().clone()),
    }))
  }
}

impl ToNapiValue for JsSourceMapObject {
  unsafe fn to_napi_value(env: sys::napi_env, value: Self) -> napi::Result<sys::napi_value> {
    unsafe { ToNapiValue::to_napi_value(env, value.0) }
  }
}

impl TypeName for JsSourceMapObject {
  fn type_name() -> &'static str {
    "JsSourceMap"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Object
  }
}

impl ValidateNapiValue for JsSourceMapObject {}
