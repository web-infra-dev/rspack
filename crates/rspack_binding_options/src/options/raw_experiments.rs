use napi_derive::napi;
use rspack_core::RspackFuture;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawIncrementalRebuild {
  pub make: bool,
  pub emit_asset: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct RawRspackFuture {}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: RawIncrementalRebuild,
  pub async_web_assembly: bool,
  pub new_split_chunks: bool,
  pub css: bool,
  pub rspack_future: RawRspackFuture,
}

impl From<RawRspackFuture> for RspackFuture {
  fn from(_value: RawRspackFuture) -> Self {
    Self {}
  }
}
