use rspack_core::{CompilerOptionsBuilder, NodeOption};
use serde::Deserialize;

#[cfg(feature = "node-api")]
use napi_derive::napi;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawNodeOption {
  pub dirname: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[cfg(not(feature = "node-api"))]
pub struct RawNodeOption {
  pub dirname: Option<String>,
}

impl RawOption<NodeOption> for RawNodeOption {
  fn to_compiler_option(self, _options: &CompilerOptionsBuilder) -> anyhow::Result<NodeOption> {
    Ok(NodeOption {
      dirname: self.dirname.unwrap_or_else(|| "false".to_string()),
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
