use napi_derive::napi;
use rspack_plugin_dts::{DtsPluginEntry, DtsPluginOptions};

#[napi(object, object_to_js = false)]
pub struct RawDtsPluginEntry {
  pub name: String,
  pub request: String,
}

#[napi(object, object_to_js = false)]
pub struct RawDtsPluginOptions {
  pub entries: Vec<RawDtsPluginEntry>,
  pub filename: String,
  pub externals: Vec<String>,
}

impl From<RawDtsPluginOptions> for DtsPluginOptions {
  fn from(value: RawDtsPluginOptions) -> Self {
    Self {
      entries: value
        .entries
        .into_iter()
        .map(|entry| DtsPluginEntry {
          name: entry.name,
          request: entry.request,
        })
        .collect(),
      filename: value.filename,
      externals: value.externals,
    }
  }
}
