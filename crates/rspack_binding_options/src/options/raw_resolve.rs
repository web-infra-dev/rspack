use crate::RawOption;
#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, Resolve};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawResolveOptions {}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawResolveOptions {}

impl RawOption<Resolve> for RawResolveOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Resolve> {
    // TODO: read resolve
    Ok(Default::default())
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
