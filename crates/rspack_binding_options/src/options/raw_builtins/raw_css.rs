use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
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
  pub preset_env: Option<Vec<String>>,

  pub modules: Option<RawCssModulesConfig>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawCssModulesConfig {
  pub locals_convention: Option<String>,
  pub local_ident_name: Option<String>,
  pub exports_only: Option<bool>,
}
