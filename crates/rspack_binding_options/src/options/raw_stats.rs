#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, StatsOptions};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawStatsOptions {
  pub colors: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawStatsOptions {
  pub colors: bool,
}

impl RawOption<StatsOptions> for RawStatsOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<StatsOptions> {
    Ok(StatsOptions {
      colors: self.colors,
    })
  }

  fn fallback_value(_: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
