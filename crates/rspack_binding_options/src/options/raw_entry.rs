use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawEntryItem {
  pub import: Vec<String>,
  pub runtime: Option<String>,
}
