use napi::bindgen_prelude::FromNapiValue;
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
pub struct RawCleanOptions(Either<bool, JsCleanOptions>);

impl FromNapiValue for RawCleanOptions {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let val = Either::from_napi_value(env, napi_val);
    Ok(Self(val?))
  }
}

/// File clean options
///
/// This matches with:
/// - keep:
///   - If a string, keep the files under this path
#[napi(object, object_to_js = false)]
#[derive(Debug)]
pub struct JsCleanOptions {
  pub keep: Option<String>,
  // todo:
  // - support RegExp type
  //   if path match the RegExp, keep the file
  // - support function type
  //    if the fn returns true on path str, keep the file
}

impl JsCleanOptions {
  pub fn to_clean_options(&self) -> CleanOptions {
    let keep = self.keep.as_ref();
    if let Some(path) = keep {
      let p = path.as_str();
      CleanOptions::from(p)
    } else {
      CleanOptions::CleanAll(false)
    }
  }
}

impl RawCleanOptions {
  pub fn to_clean_options(&self) -> CleanOptions {
    match &self.0 {
      Either::A(b) => CleanOptions::CleanAll(*b),
      Either::B(f) => f.to_clean_options(),
    }
  }
}
