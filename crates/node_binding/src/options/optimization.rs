use napi_derive::napi;
use rspack_core::BundleMode;
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

impl From<BundleMode> for RawOptimizationOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      minify: Some(mode.is_prod()),
      split_chunks: Some(mode.into()),
      remove_empty_chunks: Some(!mode.is_none()),
      chunk_id_algo: Some("named".to_string()),
      module_id_algo: Some("named".to_string()),
    }
  }
}
