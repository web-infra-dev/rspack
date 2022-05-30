use napi_derive::napi;
use rspack_core::BundleMode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawReactOptions {
  pub fast_fresh: Option<bool>,
}
