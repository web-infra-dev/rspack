use serde::Deserialize;

#[cfg(not(feature = "test"))]
use napi_derive::napi;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default = "Default::default")]
#[cfg(not(feature = "test"))]
#[napi(object)]
pub struct RawCssOptions {
  /// ## Example
  /// ```rust,ignore
  /// RawCssOptions {
  ///   preset_env: vec![
  ///
  ///          "Firefox > 10".into(),
  ///    "chrome >=20".into(),
  /// ]
  /// }
  /// ```
  pub preset_env: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default = "Default::default")]
#[cfg(feature = "test")]
pub struct RawCssOptions {
  /// ## Example
  /// ```rust,ignore
  /// RawCssOptions {
  ///   preset_env: vec!["Firefox > 10".into(), "chrome >=20".into()],
  /// }
  /// ```
  /// The preset_env will finally pass into [`browserslist::resolve`](https://docs.rs/browserslist-rs/latest/browserslist/fn.resolve.html).
  /// For detailed configuration, see https://docs.rs/browserslist-rs/latest/browserslist/
  pub preset_env: Vec<String>,
}
