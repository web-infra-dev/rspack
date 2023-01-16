use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, DevServerOptions};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawDevServer {
  pub hot: Option<bool>,
}

impl RawOption<DevServerOptions> for RawDevServer {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<DevServerOptions> {
    Ok(DevServerOptions {
      hot: self.hot.unwrap_or(false),
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
