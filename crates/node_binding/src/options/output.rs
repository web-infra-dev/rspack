use rspack_core::{BundleMode, BundleOptions};
use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[cfg(not(feature = "test"))]
pub struct RawOutputOptions {
  pub outdir: Option<String>,
  pub entry_filename: Option<String>,
  #[napi(ts_type = "\"linked\" | \"external\" | \"inline\" | \"none\"")]
  pub source_map: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "test")]
pub struct RawOutputOptions {
  pub outdir: Option<String>,
  pub entry_filename: Option<String>,
  pub source_map: Option<String>,
}

impl From<BundleMode> for RawOutputOptions {
  fn from(_mode: BundleMode) -> Self {
    Self {
      outdir: Some(BundleOptions::default().outdir),
      entry_filename: Some(BundleOptions::default().entry_filename),
      source_map: Some(if BundleOptions::default().source_map.is_enabled() {
        "inline".to_string()
      } else {
        "none".to_string()
      }),
    }
  }
}
