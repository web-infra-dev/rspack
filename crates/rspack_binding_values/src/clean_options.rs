use napi_derive::napi;
use rspack_core::CleanOptions;
use rspack_napi::napi;

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
