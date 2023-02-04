use napi_derive::napi;
use serde::Deserialize;

use crate::RawSplitChunksOptions;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOptimizationOptions {
  pub split_chunks: Option<RawSplitChunksOptions>,
  pub module_ids: Option<String>,
  pub remove_available_modules: Option<bool>,
}
