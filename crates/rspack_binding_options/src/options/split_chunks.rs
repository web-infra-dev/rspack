use napi_derive::napi;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSplitChunksOptions {
  pub code_splitting: Option<bool>,
  pub reuse_exsting_chunk: Option<bool>,
}
