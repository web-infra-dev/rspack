use napi_derive::napi;
use rspack_plugin_progress::ProgressPluginConfig;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawProgressPluginConfig {
  pub prefix: Option<String>,
}

impl From<RawProgressPluginConfig> for ProgressPluginConfig {
  fn from(value: RawProgressPluginConfig) -> Self {
    Self {
      prefix: value.prefix,
    }
  }
}
