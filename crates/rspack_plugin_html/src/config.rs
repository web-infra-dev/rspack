use serde::Deserialize;
use std::{collections::HashMap, str::FromStr};

use crate::sri::HtmlSriHashFunction;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlPluginConfigInject {
  Head,
  Body,
}

impl FromStr for HtmlPluginConfigInject {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("head") {
      Ok(HtmlPluginConfigInject::Head)
    } else if s.eq("body") {
      Ok(HtmlPluginConfigInject::Body)
    } else {
      Err(anyhow::Error::msg(
        "inject in html config only support 'head' or 'body'",
      ))
    }
  }
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlPluginConfigScriptLoading {
  Blocking,
  Defer,
  Module,
}

impl FromStr for HtmlPluginConfigScriptLoading {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("blocking") {
      Ok(HtmlPluginConfigScriptLoading::Blocking)
    } else if s.eq("defer") {
      Ok(HtmlPluginConfigScriptLoading::Defer)
    } else if s.eq("module") {
      Ok(HtmlPluginConfigScriptLoading::Module)
    } else {
      Err(anyhow::Error::msg(
        "scriptLoading in html config only support 'blocking', 'defer' or 'module'",
      ))
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct HtmlPluginConfig {
  /// emitted file name in output path
  #[serde(default = "default_filename")]
  pub filename: String,
  /// template html file
  pub template: Option<String>,
  pub template_parameters: Option<HashMap<String, String>>,
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
  pub minify: bool,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, String>>,
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
      template: None,
      template_parameters: None,
      inject: None,
      public_path: None,
      script_loading: default_script_loading(),
      chunks: None,
      excluded_chunks: None,
      sri: None,
      minify: false,
      title: None,
      favicon: None,
      meta: None,
    }
  }
}
