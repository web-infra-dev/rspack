use std::str::FromStr;

use napi_derive::napi;
use rspack_plugin_css::plugin::{LocalIdentName, LocalsConvention, ModulesConfig};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssPluginConfig {
  /// ## Example
  /// ```rust,ignore
  /// RawCssOptions {
  ///   preset_env: vec!["Firefox > 10".into(), "chrome >=20".into()],
  /// }
  /// ```
  /// The preset_env will finally pass into [`browserslist::resolve`](https://docs.rs/browserslist-rs/latest/browserslist/fn.resolve.html).
  /// For detailed configuration, see https://docs.rs/browserslist-rs/latest/browserslist/
  pub preset_env: Vec<String>,

  pub modules: RawCssModulesConfig,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssModulesConfig {
  pub locals_convention: String,
  pub local_ident_name: String,
  pub exports_only: bool,
}

impl TryFrom<RawCssModulesConfig> for ModulesConfig {
  type Error = rspack_error::Error;

  fn try_from(value: RawCssModulesConfig) -> Result<Self, Self::Error> {
    Ok(Self {
      locals_convention: LocalsConvention::from_str(&value.locals_convention)?,
      local_ident_name: LocalIdentName::from(value.local_ident_name),
      exports_only: value.exports_only,
    })
  }
}
