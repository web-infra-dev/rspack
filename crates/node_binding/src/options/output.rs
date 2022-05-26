use napi_derive::napi;
use rspack_core::{BundleMode, BundleOptions};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawOutputOptions {
  pub outdir: Option<String>,
  pub entry_filename: Option<String>,
  pub source_map: Option<bool>,
}

impl From<BundleMode> for RawOutputOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      outdir: Some(BundleOptions::default().outdir),
      entry_filename: Some(BundleOptions::default().entry_filename),
      source_map: Some(BundleOptions::default().source_map),
    }
  }
}
