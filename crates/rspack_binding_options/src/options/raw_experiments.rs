use std::str::FromStr;

use napi_derive::napi;
use rspack_plugin_css::plugin::{LocalIdentName, LocalsConvention, ModulesConfig};
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawIncrementalRebuild {
  pub make: bool,
  pub emit_asset: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssExperimentOptions {
  #[napi(ts_type = "\"asIs\" | \"camelCase\" | \"camelCaseOnly\" | \"dashes\" | \"dashesOnly\"")]
  pub locals_convention: String,
  pub local_ident_name: String,
  pub exports_only: bool,
}

impl TryFrom<RawCssExperimentOptions> for ModulesConfig {
  type Error = rspack_error::Error;

  fn try_from(value: RawCssExperimentOptions) -> Result<Self, Self::Error> {
    Ok(Self {
      locals_convention: LocalsConvention::from_str(&value.locals_convention)?,
      local_ident_name: LocalIdentName::from(value.local_ident_name),
      exports_only: value.exports_only,
    })
  }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawExperiments {
  pub lazy_compilation: bool,
  pub incremental_rebuild: RawIncrementalRebuild,
  pub async_web_assembly: bool,
  pub new_split_chunks: bool,
  pub css: Option<RawCssExperimentOptions>,
}
