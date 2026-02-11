use napi_derive::napi;
use rspack_plugin_extract_css::plugin::{CssExtractOptions, InsertType};
use rustc_hash::FxHashMap as HashMap;

use crate::filename::JsFilename;

#[napi(object, object_to_js = false)]
pub struct RawCssExtractPluginOption {
  pub filename: JsFilename,
  pub chunk_filename: JsFilename,
  pub ignore_order: bool,
  pub insert: Option<String>,
  pub attributes: HashMap<String, String>,
  pub link_type: Option<String>,
  pub runtime: bool,
  pub pathinfo: bool,
  pub enforce_relative: bool,
}

impl From<RawCssExtractPluginOption> for CssExtractOptions {
  fn from(value: RawCssExtractPluginOption) -> Self {
    Self {
      filename: value.filename.into(),
      chunk_filename: value.chunk_filename.into(),
      ignore_order: value.ignore_order,
      insert: value.insert.map_or(InsertType::Default, |insert| {
        if insert.starts_with("function") || insert.starts_with('(') {
          InsertType::Fn(insert)
        } else {
          InsertType::Selector(insert)
        }
      }),
      attributes: value.attributes.into_iter().collect(),
      link_type: value.link_type,
      runtime: value.runtime,
      pathinfo: value.pathinfo,
      enforce_relative: value.enforce_relative,
    }
  }
}
