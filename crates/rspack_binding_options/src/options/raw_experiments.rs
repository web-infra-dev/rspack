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
pub struct RawRspackFuture {
  pub new_treeshaking: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub incremental_rebuild: RawIncrementalRebuild,
  pub new_split_chunks: bool,
  pub top_level_await: bool,
  pub rspack_future: RawRspackFuture,
}

impl From<RawRspackFuture> for RspackFuture {
  fn from(value: RawRspackFuture) -> Self {
    Self {
      new_treeshaking: value.new_treeshaking,
    }
  }
}
