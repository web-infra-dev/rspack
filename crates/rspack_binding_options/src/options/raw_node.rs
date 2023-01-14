use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, NodeOption, TargetPlatform};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawNodeOption {
  pub dirname: Option<String>,
}

impl RawOption<NodeOption> for RawNodeOption {
  fn to_compiler_option(self, options: &CompilerOptionsBuilder) -> anyhow::Result<NodeOption> {
    Ok(NodeOption {
      dirname: self.dirname.unwrap_or_else(|| {
        if let Some(target) = &options.target {
          if matches!(target.platform, TargetPlatform::Web) {
            return "mock".to_string();
          }
        }
        "false".to_string()
      }),
    })
  }

  fn fallback_value(_options: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
