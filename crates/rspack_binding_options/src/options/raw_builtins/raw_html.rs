#[cfg(feature = "node-api")]
use napi_derive::napi;

use rspack_plugin_html::config::HtmlPluginConfig;
use rspack_plugin_html::config::HtmlPluginConfigInject;
use rspack_plugin_html::config::HtmlPluginConfigScriptLoading;
use rspack_plugin_html::sri::HtmlSriHashFunction;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

use crate::RawOption;

pub type RawHtmlPluginConfigScriptLoading = String;
pub type RawHtmlPluginConfigInject = String;
pub type RawHtmlSriHashFunction = String;
pub type RawHtmlFilename = String;

// fn default_template() -> String {
//   String::from("index.html")
// }

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
#[cfg(feature = "node-api")]
#[napi(object)]
pub struct RawHtmlPluginConfig {
  /// emitted file name in output path
  #[napi(ts_type = "string | void")]
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
  /// `head`, `body` or None
  #[napi(ts_type = "string | void")]
  pub inject: Option<RawHtmlPluginConfigInject>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[napi(ts_type = "string | void")]
  pub script_loading: Option<RawHtmlPluginConfigScriptLoading>,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,
  #[napi(ts_type = "string | void")]
  pub sri: Option<RawHtmlSriHashFunction>,
  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Serialize)]
#[cfg(not(feature = "node-api"))]
pub struct RawHtmlPluginConfig {
  /// emitted file name in output path
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
  /// `head`, `body` or None
  pub inject: Option<RawHtmlPluginConfigInject>,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  pub script_loading: Option<RawHtmlPluginConfigScriptLoading>,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub excluded_chunks: Option<Vec<String>>,
  pub sri: Option<RawHtmlSriHashFunction>,
  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, String>>,
}

impl RawOption<HtmlPluginConfig> for RawHtmlPluginConfig {
  fn to_compiler_option(
    self,
    _options: &rspack_core::CompilerOptionsBuilder,
  ) -> anyhow::Result<HtmlPluginConfig> {
    let inject = self
      .inject
      .as_ref()
      .map(|s| HtmlPluginConfigInject::from_str(s).unwrap());

    let script_loading = HtmlPluginConfigScriptLoading::from_str(
      &self.script_loading.unwrap_or_else(|| String::from("defer")),
    )?;

    let sri = self
      .sri
      .as_ref()
      .map(|s| HtmlSriHashFunction::from_str(s).unwrap());

    Ok(HtmlPluginConfig {
      filename: self.filename.unwrap_or_else(|| String::from("index.html")),
      template: self.template,
      inject,
      public_path: self.public_path,
      script_loading,
      chunks: self.chunks,
      excluded_chunks: self.excluded_chunks,
      sri,
      minify: self.minify.unwrap_or_default(),
      title: self.title,
      favicon: self.favicon,
      meta: self.meta,
    })
  }

  fn fallback_value(_options: &rspack_core::CompilerOptionsBuilder) -> Self {
    Self {
      filename: Some(String::from("index.html")),
      template: Default::default(),
      inject: Default::default(),
      public_path: Default::default(),
      script_loading: Some(String::from("defer")),
      chunks: Default::default(),
      excluded_chunks: Default::default(),
      sri: Default::default(),
      minify: Default::default(),
      title: Default::default(),
      favicon: Default::default(),
      meta: None,
    }
  }
}
