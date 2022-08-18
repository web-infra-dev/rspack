use napi_derive::napi;
use serde::Deserialize;
use serde::Serialize;
pub type HtmlPluginConfigScriptLoading = String;
pub type HtmlPluginConfigInject = String;
pub type HtmlSriHashFunction = String;

// fn default_template() -> String {
//   String::from("index.html")
// }
fn default_filename() -> String {
  String::from("index.html")
}

fn default_script_loading() -> HtmlPluginConfigScriptLoading {
  String::from("defer")
}

/**
 * It seems napi not support enum well
 */
// #[derive(Deserialize, Debug, Clone, Copy)]
// #[serde(rename_all = "snake_case")]
// pub enum HtmlSriHashFunction {
//   Sha256,
//   Sha384,
//   Sha512,
// }
// #[derive(Deserialize, Debug, Serialize)]
// #[serde(rename_all = "snake_case")]
// #[napi]
// pub enum HtmlPluginConfigInject {
//   Head,
//   Body,
// }
// #[derive(Deserialize, Debug, Serialize)]
// #[serde(rename_all = "snake_case")]
// #[napi]
// pub enum HtmlPluginConfigScriptLoading {
//   Blocking,
//   Defer,
//   Module,
// }

#[derive(Deserialize, Debug, Serialize)]
#[napi(object)]
pub struct RawHtmlPluginConfig {
  /// emitted file name in output path
  #[serde(default = "default_filename")]
  pub filename: String,
  /// template html file
  pub template: Option<String>,
  /// `head`, `body` or None
  #[napi(ts_type = "string | void")]
  pub inject: Option<HtmlPluginConfigInject>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[serde(default = "default_script_loading")]
  #[napi(ts_type = "string | void")]
  pub script_loading: HtmlPluginConfigScriptLoading,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,
  #[napi(ts_type = "string | void")]
  pub sri: Option<HtmlSriHashFunction>,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
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
}

#[derive(Debug, Deserialize, Default)]
#[napi(object)]
pub struct RawBuiltins {
  pub html: Option<Vec<RawHtmlPluginConfig>>,
  pub css: Option<RawCssPluginConfig>,
}
