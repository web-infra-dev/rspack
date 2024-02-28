use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::Filename;
use rspack_plugin_extract_css::plugin::{CssExtractOptions, InsertType};

#[napi(object)]
pub struct RawCssExtractPluginOption {
  pub filename: String,
  pub chunk_filename: String,
  pub ignore_order: bool,
  pub insert: Option<String>,
  pub attributes: HashMap<String, String>,
  pub link_type: Option<String>,
  pub runtime: bool,
  pub pathinfo: bool,
}

impl From<RawCssExtractPluginOption> for CssExtractOptions {
  fn from(value: RawCssExtractPluginOption) -> Self {
    Self {
      filename: Filename::from(value.filename),
      chunk_filename: Filename::from(value.chunk_filename),
      ignore_order: value.ignore_order,
      insert: value
        .insert
        .map(|insert| {
          if insert.starts_with("function") || insert.starts_with('(') {
            InsertType::Fn(insert)
          } else {
            InsertType::Selector(insert)
          }
        })
        .unwrap_or(InsertType::Default),
      attributes: value.attributes.into_iter().collect(),
      link_type: value.link_type,
      runtime: value.runtime,
      pathinfo: value.pathinfo,
    }
  }
}
