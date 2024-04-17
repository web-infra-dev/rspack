use std::fmt::Debug;
use std::sync::Arc;

use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue, ValidateNapiValue},
  Either, JsFunction,
};
use rspack_core::{AssetInfo, LocalFilenameFn, PathData};
use rspack_core::{Filename, FilenameFn};
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use serde::Deserialize;

use crate::{JsAssetInfo, JsPathData};

/// A js filename value. Either a string or a function
///
/// The function type is generic. By default the function type is tsfn.
#[derive(Debug)]
pub struct JsFilename<F = ThreadsafeFunction<(JsPathData, Option<JsAssetInfo>), String>>(
  Either<String, F>,
);

/// A local js filename value. Only valid in the current native call.
///
/// Useful as the type of a parameter that is invoked immediately inside the function.
pub type LocalJsFilename = JsFilename<JsFunction>;

impl From<LocalJsFilename> for Filename<LocalJsFilenameFn> {
  fn from(value: LocalJsFilename) -> Self {
    match value.0 {
      Either::A(template) => Filename::from(template),
      Either::B(js_func) => Filename::from_fn(LocalJsFilenameFn(js_func)),
    }
  }
}
impl From<JsFilename> for Filename {
  fn from(value: JsFilename) -> Self {
    match value.0 {
      Either::A(template) => Filename::from(template),
      Either::B(theadsafe_filename_fn) => {
        Filename::from_fn(Arc::new(ThreadSafeFilenameFn(theadsafe_filename_fn)))
      }
    }
  }
}

impl<F: FromNapiValue + ValidateNapiValue> FromNapiValue for JsFilename<F> {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    Ok(Self(Either::from_napi_value(env, napi_val)?))
  }
}

impl<F: ToNapiValue> ToNapiValue for JsFilename<F> {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    Either::to_napi_value(env, val.0)
  }
}

impl<'de> Deserialize<'de> for JsFilename {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    Ok(Self(Either::A(String::deserialize(deserializer)?)))
  }
}

/// Wrapper of a thread-safe filename js function. Implements `FilenameFn`
#[derive(Debug)]
struct ThreadSafeFilenameFn(ThreadsafeFunction<(JsPathData, Option<JsAssetInfo>), String>);
impl LocalFilenameFn for ThreadSafeFilenameFn {
  type Error = rspack_error::Error;
  fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> rspack_error::Result<String> {
    self.0.blocking_call_with_sync((
      JsPathData::from(*path_data),
      asset_info.cloned().map(JsAssetInfo::from),
    ))
  }
}
impl FilenameFn for ThreadSafeFilenameFn {}

/// Wrapper of a local filename js function. Implements `LocalFilenameFn`. Only valid in the current native call.
pub struct LocalJsFilenameFn(JsFunction);

impl LocalFilenameFn for LocalJsFilenameFn {
  type Error = napi::Error;

  fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&AssetInfo>,
  ) -> Result<String, Self::Error> {
    let js_path_data = JsPathData::from(*path_data);
    let js_asset_info = asset_info.cloned().map(JsAssetInfo::from);
    self.0.call2(js_path_data, js_asset_info)
  }
}
