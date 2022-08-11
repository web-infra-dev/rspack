use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(feature = "test"), napi(object))]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub asset_module_filename: Option<String>,
  // pub entry_filename: Option<String>,
  // pub source_map: Option<String>,
}
