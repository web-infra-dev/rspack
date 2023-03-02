use std::path::PathBuf;

use napi_derive::napi;
use rspack_core::{RelayConfig, RelayLanguageConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawRelayConfig {
  pub artifact_directory: Option<String>,
  #[napi(ts_type = "'javascript' | 'typescript' | 'flow'")]
  pub language: String,
}

impl From<RawRelayConfig> for RelayConfig {
  fn from(raw_config: RawRelayConfig) -> Self {
    Self {
      artifact_directory: raw_config.artifact_directory.map(PathBuf::from),
      language: match raw_config.language.as_str() {
        "typescript" => RelayLanguageConfig::TypeScript,
        "flow" => RelayLanguageConfig::Flow,
        _ => RelayLanguageConfig::JavaScript,
      },
    }
  }
}
