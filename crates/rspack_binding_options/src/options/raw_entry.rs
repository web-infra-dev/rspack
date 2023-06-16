use napi_derive::napi;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawEntryDescription {
  pub import: Vec<String>,
  pub runtime: Option<String>,
  pub chunk_loading: Option<String>,
  pub public_path: Option<String>,
}
