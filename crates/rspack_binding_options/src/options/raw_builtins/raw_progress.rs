#[cfg(feature = "node-api")]
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawProgressPluginConfig {
  pub prefix: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawProgressPluginConfig {
  pub prefix: Option<String>,
}
