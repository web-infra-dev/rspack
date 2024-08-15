use std::{collections::HashMap, path::PathBuf, str::FromStr};

use derivative::Derivative;
use futures::future::BoxFuture;
use rspack_core::{Compilation, PublicPath};
use rspack_error::Result;
use serde::Deserialize;
use sugar_path::SugarPath;

use crate::sri::HtmlSriHashFunction;

#[cfg_attr(feature = "testing", derive(JsonSchema))]
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlInject {
  Head,
  Body,
  False,
}

impl FromStr for HtmlInject {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("head") {
      Ok(HtmlInject::Head)
    } else if s.eq("body") {
      Ok(HtmlInject::Body)
    } else if s.eq("false") {
      Ok(HtmlInject::False)
    } else {
      Err(anyhow::Error::msg(
        "inject in html config only support 'head', 'body', or 'false'",
      ))
    }
  }
}

#[cfg_attr(feature = "testing", derive(JsonSchema))]
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum HtmlScriptLoading {
  Blocking,
  Defer,
  Module,
  SystemjsModule,
}

impl FromStr for HtmlScriptLoading {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("blocking") {
      Ok(HtmlScriptLoading::Blocking)
    } else if s.eq("defer") {
      Ok(HtmlScriptLoading::Defer)
    } else if s.eq("module") {
      Ok(HtmlScriptLoading::Module)
    } else if s.eq("systemjs-module") {
      Ok(HtmlScriptLoading::SystemjsModule)
    } else {
      Err(anyhow::Error::msg(
        "scriptLoading in html config only support 'blocking', 'defer' or 'module'",
      ))
    }
  }
}

#[derive(Debug)]
pub struct HtmlRspackPluginBaseOptions {
  pub href: Option<String>,
  pub target: Option<String>,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct HtmlRspackPluginOptions {
  pub filename: String,
  /// template html file
  pub template: Option<String>,
  pub template_content: Option<String>,
  pub template_parameters: Option<HashMap<String, String>>,
  pub inject: HtmlInject,
  /// path or `auto`
  pub public_path: Option<String>,
  pub script_loading: HtmlScriptLoading,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub exclude_chunks: Option<Vec<String>>,

  /// hash func that used in subsource integrity
  /// sha384, sha256 or sha512
  pub sri: Option<HtmlSriHashFunction>,
  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, HashMap<String, String>>>,
  pub hash: Option<bool>,
  pub base: Option<HtmlRspackPluginBaseOptions>,
  #[derivative(Debug = "ignore")]
  pub internal_template_compile_fn: Option<TemplateCompileFn>,
}

type TemplateCompileFn =
  Box<dyn Fn(Vec<String>) -> BoxFuture<'static, Result<HashMap<String, String>>> + Sync + Send>;

fn default_filename() -> String {
  String::from("index.html")
}

fn default_script_loading() -> HtmlScriptLoading {
  HtmlScriptLoading::Defer
}

fn default_inject() -> HtmlInject {
  HtmlInject::Head
}

impl Default for HtmlRspackPluginOptions {
  fn default() -> HtmlRspackPluginOptions {
    HtmlRspackPluginOptions {
      filename: default_filename(),
      template: None,
      template_content: None,
      template_parameters: None,
      inject: default_inject(),
      public_path: None,
      script_loading: default_script_loading(),
      chunks: None,
      exclude_chunks: None,
      sri: None,
      minify: None,
      title: None,
      favicon: None,
      meta: None,
      hash: None,
      base: None,
      internal_template_compile_fn: None,
    }
  }
}

impl HtmlRspackPluginOptions {
  pub fn get_public_path(&self, compilation: &Compilation, filename: &str) -> String {
    match &self.public_path {
      Some(p) => PublicPath::ensure_ends_with_slash(p.clone()),
      None => compilation
        .options
        .output
        .public_path
        .render(compilation, filename),
    }
  }
  pub fn get_relative_path(&self, compilation: &Compilation, filename: &str) -> String {
    let mut file_path = PathBuf::from(filename);

    if file_path.is_absolute() {
      let context_path = PathBuf::from(compilation.options.context.to_string());
      file_path = file_path.relative(context_path);
    }

    file_path.to_string_lossy().to_string()
  }
}
