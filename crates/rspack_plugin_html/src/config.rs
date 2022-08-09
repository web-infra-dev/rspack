use serde::Deserialize;

use crate::sri::HtmlSriHashFunction;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlPluginConfigInject {
  Head,
  Body,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlPluginConfigScriptLoading {
  Blocking,
  Defer,
  Module,
}

#[derive(Deserialize, Debug)]
pub struct HtmlPluginConfig {
  /// emitted file name in output path
  #[serde(default = "default_filename")]
  pub filename: String,
  /// template html file
  #[serde(default = "default_template")]
  pub template: String,
  /// `head`, `body` or None
  pub inject: Option<HtmlPluginConfigInject>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[serde(default = "default_script_loading")]
  pub script_loading: HtmlPluginConfigScriptLoading,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,

  /// hash func that used in subsource integrity
  /// sha384, sha256 or sha512
  pub sri: Option<HtmlSriHashFunction>,
}

fn default_filename() -> String {
  String::from("index.html")
}

fn default_template() -> String {
  String::from("index.html")
}

fn default_script_loading() -> HtmlPluginConfigScriptLoading {
  HtmlPluginConfigScriptLoading::Defer
}

impl Default for HtmlPluginConfig {
  fn default() -> HtmlPluginConfig {
    HtmlPluginConfig {
      filename: default_filename(),
      template: default_template(),
      inject: None,
      public_path: None,
      script_loading: default_script_loading(),
      chunks: None,
      excluded_chunks: None,
      sri: None,
    }
  }
}
