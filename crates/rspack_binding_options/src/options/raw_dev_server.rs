use napi_derive::napi;
use rspack_core::DevServerOptions;
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawDevServer {
  pub hot: bool,
}

impl From<RawDevServer> for DevServerOptions {
  fn from(value: RawDevServer) -> Self {
    Self { hot: value.hot }
  }
}
