#[cfg(feature = "node-api")]
use napi_derive::napi;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawCssPluginConfig {
  /// ## Example
  /// ```rust,ignore
  /// RawCssOptions {
  ///   preset_env: vec!["Firefox > 10".into(), "chrome >=20".into()],
  /// }
  /// ```
  /// The preset_env will finally pass into [`browserslist::resolve`](https://docs.rs/browserslist-rs/latest/browserslist/fn.resolve.html).
  /// For detailed configuration, see https://docs.rs/browserslist-rs/latest/browserslist/
  pub preset_env: Option<Vec<String>>,

  pub modules: Option<RawCssModulesConfig>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawCssPluginConfig {
  /// ## Example
  /// ```rust,ignore
  /// RawCssOptions {
  ///   preset_env: vec!["Firefox > 10".into(), "chrome >=20".into()],
  /// }
  /// ```
  /// The preset_env will finally pass into [`browserslist::resolve`](https://docs.rs/browserslist-rs/latest/browserslist/fn.resolve.html).
  /// For detailed configuration, see https://docs.rs/browserslist-rs/latest/browserslist/
  pub preset_env: Option<Vec<String>>,

  pub modules: Option<RawCssModulesConfig>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(feature = "node-api")]
#[napi(object)]
#[serde(rename_all = "camelCase")]
pub struct RawCssModulesConfig {
  pub locals_convention: Option<String>,
  pub local_ident_name: Option<String>,
  pub exports_only: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[cfg(not(feature = "node-api"))]
#[serde(rename_all = "camelCase")]
pub struct RawCssModulesConfig {
  pub locals_convention: Option<String>,
  pub local_ident_name: Option<String>,
  pub exports_only: Option<bool>,
}
