use napi_derive::napi;
use rspack_core::StatsOptions;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawStatsOptions {
  pub colors: bool,
}

impl From<RawStatsOptions> for StatsOptions {
  fn from(value: RawStatsOptions) -> Self {
    Self {
      colors: value.colors,
    }
  }
}
