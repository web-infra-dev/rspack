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
}

impl From<BundleMode> for RawOptimizationOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      minify: Some(mode.is_prod()),
      split_chunks: Some(mode.into()),
    }
  }
}
