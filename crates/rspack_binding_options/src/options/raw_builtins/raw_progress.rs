use napi_derive::napi;
use rspack_plugin_progress::ProgressPluginOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawProgressPluginOptions {
  pub prefix: String,
  pub profile: bool,
}

impl From<RawProgressPluginOptions> for ProgressPluginOptions {
  fn from(value: RawProgressPluginOptions) -> Self {
    Self {
      prefix: value.prefix,
      profile: value.profile,
    }
  }
}
