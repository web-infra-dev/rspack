use std::collections::HashMap;
use std::str::FromStr;

use napi_derive::napi;
use rspack_plugin_html::config::HtmlPluginConfig;
use rspack_plugin_html::config::HtmlPluginConfigInject;
use rspack_plugin_html::config::HtmlPluginConfigScriptLoading;
use rspack_plugin_html::sri::HtmlSriHashFunction;
use serde::Deserialize;
use serde::Serialize;

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
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawHtmlPluginConfig {
  /// emitted file name in output path
  #[napi(ts_type = "string")]
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
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

impl RawOption<HtmlPluginConfig> for RawHtmlPluginConfig {
  fn to_compiler_option(
    self,
    _options: &rspack_core::CompilerOptionsBuilder,
  ) -> anyhow::Result<HtmlPluginConfig> {
    let inject = self.inject.as_ref().map(|s| {
      HtmlPluginConfigInject::from_str(s).unwrap_or_else(|_| panic!("Invalid inject value: {s}"))
    });

    let script_loading = HtmlPluginConfigScriptLoading::from_str(
      &self.script_loading.unwrap_or_else(|| String::from("defer")),
    )?;

    let sri = self.sri.as_ref().map(|s| {
      HtmlSriHashFunction::from_str(s).unwrap_or_else(|_| panic!("Invalid sri value: {s}"))
    });

    Ok(HtmlPluginConfig {
      filename: self.filename.unwrap_or_else(|| String::from("index.html")),
      template: self.template,
      template_parameters: self.template_parameters,
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
      template_parameters: None,
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
