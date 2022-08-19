use crate::{define_napi_object, RawOption};
#[cfg(feature = "node-api")]
use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, Resolve};
use serde::Deserialize;

define_napi_object!(
  #[derive(Deserialize, Debug, Default)]
  #[serde(rename_all = "camelCase")]
  pub struct RawResolveOptions {}
);

impl RawOption<Resolve> for RawResolveOptions {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<Resolve> {
    // TODO: read resolve
    Ok(Default::default())
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
