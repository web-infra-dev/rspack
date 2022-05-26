use napi_derive::napi;
use rspack_core::BundleMode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactOptions {
  pub fast_fresh: Option<bool>,
}

impl From<BundleMode> for RawReactOptions {
  fn from(mode: BundleMode) -> Self {
    Self {
      fast_fresh: Some(mode.is_dev() && !mode.is_none()),
    }
  }
}
