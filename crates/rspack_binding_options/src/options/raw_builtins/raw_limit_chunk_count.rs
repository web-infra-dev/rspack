use napi_derive::napi;
use rspack_plugin_limit_chunk_count::LimitChunkCountPluginOptions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawLimitChunkCountPluginOptions {}

impl From<RawLimitChunkCountPluginOptions> for LimitChunkCountPluginOptions {
  fn from(value: RawLimitChunkCountPluginOptions) -> Self {
    Self {}
  }
}
