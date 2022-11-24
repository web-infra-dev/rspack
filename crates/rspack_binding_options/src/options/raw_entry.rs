#[cfg(feature = "node-api")]
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawEntryItem {
  pub import: Vec<String>,
  pub runtime: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawEntryItem {
  pub import: Vec<String>,
  pub runtime: String,
}
