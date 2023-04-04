use std::collections::HashMap;
use std::str::FromStr;

use napi_derive::napi;
use rspack_plugin_html::config::HtmlPluginConfig;
use rspack_plugin_html::config::HtmlPluginConfigInject;
use rspack_plugin_html::config::HtmlPluginConfigScriptLoading;
use rspack_plugin_html::sri::HtmlSriHashFunction;
use serde::Deserialize;
use serde::Serialize;

pub type RawHtmlPluginConfigScriptLoading = String;
pub type RawHtmlPluginConfigInject = String;
pub type RawHtmlSriHashFunction = String;
pub type RawHtmlFilename = String;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawHtmlPluginConfig {
  /// emitted file name in output path
  #[napi(ts_type = "string")]
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
  pub template_content: Option<String>,
  pub template_parameters: Option<HashMap<String, String>>,
  /// `head`, `body` or None
  #[napi(ts_type = "\"head\" | \"body\"")]
  pub inject: Option<RawHtmlPluginConfigInject>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[napi(ts_type = "\"blocking\" | \"defer\" | \"module\"")]
  pub script_loading: Option<RawHtmlPluginConfigScriptLoading>,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,
  #[napi(ts_type = "\"sha256\" | \"sha384\" | \"sha512\"")]
  pub sri: Option<RawHtmlSriHashFunction>,
  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, HashMap<String, String>>>,
}

impl From<RawHtmlPluginConfig> for HtmlPluginConfig {
  fn from(value: RawHtmlPluginConfig) -> Self {
    let inject = value.inject.as_ref().map(|s| {
      HtmlPluginConfigInject::from_str(s).unwrap_or_else(|_| panic!("Invalid inject value: {s}"))
    });

    let script_loading = HtmlPluginConfigScriptLoading::from_str(
      &value
        .script_loading
        .unwrap_or_else(|| String::from("defer")),
    )
    .expect("value.script_loading has unwrap_or_else so this will never happen");

    let sri = value.sri.as_ref().map(|s| {
      HtmlSriHashFunction::from_str(s).unwrap_or_else(|_| panic!("Invalid sri value: {s}"))
    });

    HtmlPluginConfig {
      filename: value.filename.unwrap_or_else(|| String::from("index.html")),
      template: value.template,
      template_content: value.template_content,
      template_parameters: value.template_parameters,
      inject,
      public_path: value.public_path,
      script_loading,
      chunks: value.chunks,
      excluded_chunks: value.excluded_chunks,
      sri,
      minify: value.minify.unwrap_or_default(),
      title: value.title,
      favicon: value.favicon,
      meta: value.meta,
    }
  }
}
