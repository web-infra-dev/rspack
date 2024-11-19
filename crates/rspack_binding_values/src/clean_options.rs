use std::path::Path;

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue};
use napi::Either;
use napi_derive::napi;
use rspack_core::CleanOptions;
use rspack_napi::napi;

/// File clean options
///
/// A file filter is an option whether the file should be kept after clean up
///
/// TS Type:
///
/// ```typescript
/// // in the future, we should support the following types, just like webpack
/// // type CleanOptions = boolean | { dry?: boolean, keep?: RegExp | string | ((filename: string) => boolean) }
///
/// type CleanOptions = boolean | { keep?: string }
/// ```
#[derive(Debug)]
pub struct JsCleanOptions(Either<bool, JsCleanFilter>);

impl ToNapiValue for JsCleanOptions {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    val: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    Either::to_napi_value(env, val.0)
  }
}

impl FromNapiValue for JsCleanOptions {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    Ok(Self(Either::from_napi_value(env, napi_val)?))
  }
}

/// File clean filter object
///
/// This clean filter matches with:
/// - keep:
///   - If a string, keep the files under this path
#[napi(object)]
#[derive(Debug)]
pub struct JsCleanFilter {
  pub keep: Option<String>,
  // todo:
  // - support RegExp type
  //   if path match the RegExp, keep the file
  // - support function type
  //    if the fn returns true on path str, keep the file
}

impl JsCleanFilter {
  pub fn to_clean_options<T: AsRef<Path>>(&self, working_dir: T) -> CleanOptions {
    let wd = working_dir.as_ref();
    let keep = self.keep.as_ref().map(|p| wd.join(p));
    if let Some(path) = keep {
      CleanOptions::KeepPath(path)
    } else {
      CleanOptions::Boolean(false)
    }
  }
}

impl JsCleanOptions {
  pub fn to_clean_options<T: AsRef<Path>>(&self, working_dir: T) -> CleanOptions {
    match &self.0 {
      Either::A(b) => CleanOptions::Boolean(*b),
      Either::B(f) => f.to_clean_options(working_dir),
    }
  }
}
