use std::collections::HashMap;
use std::str::FromStr;

use napi_derive::napi;
use rspack_plugin_html::config::HtmlInject;
use rspack_plugin_html::config::HtmlRspackPluginOptions;
use rspack_plugin_html::config::HtmlScriptLoading;
use rspack_plugin_html::sri::HtmlSriHashFunction;

pub type RawHtmlScriptLoading = String;
pub type RawHtmlInject = String;
pub type RawHtmlSriHashFunction = String;
pub type RawHtmlFilename = String;

#[derive(Debug)]
#[napi(object)]
pub struct RawHtmlRspackPluginOptions {
  /// emitted file name in output path
  #[napi(ts_type = "string")]
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
  pub template_content: Option<String>,
  pub template_parameters: Option<HashMap<String, String>>,
  /// "head", "body" or "false"
  #[napi(ts_type = "\"head\" | \"body\" | \"false\"")]
  pub inject: RawHtmlInject,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[napi(ts_type = "\"blocking\" | \"defer\" | \"module\"")]
  pub script_loading: RawHtmlScriptLoading,

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

impl From<RawHtmlRspackPluginOptions> for HtmlRspackPluginOptions {
  fn from(value: RawHtmlRspackPluginOptions) -> Self {
    let inject = HtmlInject::from_str(&value.inject).expect("Invalid inject value");

    let script_loading =
      HtmlScriptLoading::from_str(&value.script_loading).expect("Invalid script_loading value");

    let sri = value.sri.as_ref().map(|s| {
      HtmlSriHashFunction::from_str(s).unwrap_or_else(|_| panic!("Invalid sri value: {s}"))
    });

    HtmlRspackPluginOptions {
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
