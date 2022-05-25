use napi_derive::napi;
use rspack_core::BundleMode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSplitChunksOptions {
  pub code_splitting: Option<bool>,
  pub reuse_exsting_chunk: Option<bool>,
}

impl From<BundleMode> for RawSplitChunksOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      code_splitting: Some(!mode.is_none()),
      reuse_exsting_chunk: Some(!mode.is_none()),
    }
  }
}
