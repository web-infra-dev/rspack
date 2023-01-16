use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawProgressPluginConfig {
  pub prefix: Option<String>,
}
