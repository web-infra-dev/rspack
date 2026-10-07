use napi_derive::napi;
use rspack_core::StatsOptions;

#[derive(Debug, Default)]
#[napi(object)]
pub struct RawStatsOptions {
  pub colors: bool,
  pub cache_info: bool,
}

impl From<RawStatsOptions> for StatsOptions {
  fn from(value: RawStatsOptions) -> Self {
    Self {
      colors: value.colors,
      cache_info: value.cache_info,
    }
  }
}
