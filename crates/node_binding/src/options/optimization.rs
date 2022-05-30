use napi_derive::napi;
use serde::Deserialize;

use crate::RawSplitChunksOptions;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub split_chunks: Option<RawSplitChunksOptions>,
  pub minify: Option<bool>,
  pub remove_empty_chunks: Option<bool>,
  pub chunk_id_algo: Option<String>,
  pub module_id_algo: Option<String>,
}
