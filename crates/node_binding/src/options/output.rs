use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[cfg(not(feature = "test"))]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub entry_filename: Option<String>,
  #[napi(ts_type = "\"linked\" | \"external\" | \"inline\" | \"none\"")]
  pub source_map: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawOutputOptions {
  pub path: Option<String>,
  pub entry_filename: Option<String>,
  pub source_map: Option<String>,
}
