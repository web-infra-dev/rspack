use std::{fmt, path::PathBuf, str::FromStr};

use futures::future::BoxFuture;
use rspack_core::{Compilation, PublicPath};
use rspack_error::Result;
use rspack_util::fx_hash::FxHashMap;
use serde::Serialize;
use sugar_path::SugarPath;

#[derive(Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum HtmlInject {
  #[default]
  Head,
  Body,
  False,
}

impl fmt::Display for HtmlInject {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      HtmlInject::Head => "head",
      HtmlInject::Body => "body",
      HtmlInject::False => "false",
    })
  }
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

#[derive(Serialize, Debug, Clone, Copy)]
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

type TemplateParameterTsfn =
  Box<dyn for<'a> Fn(String) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub struct TemplateParameterFn {
  pub inner: TemplateParameterTsfn,
}

impl std::fmt::Debug for TemplateParameterFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TemplateParameterFn").finish()
  }
}

#[derive(Debug)]
pub enum TemplateParameters {
  Map(FxHashMap<String, String>),
  Function(TemplateParameterFn),
  Disabled,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HtmlRspackPluginBaseOptions {
  pub href: Option<String>,
  pub target: Option<String>,
}

type TemplateRenderTsfn =
  Box<dyn for<'a> Fn(String) -> BoxFuture<'static, Result<String>> + Sync + Send>;

pub struct TemplateRenderFn {
  pub inner: TemplateRenderTsfn,
}

impl std::fmt::Debug for TemplateRenderFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TemplateRenderFn").finish()
  }
}

#[derive(Serialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum HtmlChunkSortMode {
  #[default]
  Auto,
  Manual,
  // TODO: support function
}

impl FromStr for HtmlChunkSortMode {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.eq("auto") {
      Ok(HtmlChunkSortMode::Auto)
    } else if s.eq("manual") {
      Ok(HtmlChunkSortMode::Manual)
    } else {
      Err(anyhow::Error::msg(
        "chunksSortMode in html config only support 'auto' or 'manual'",
      ))
    }
  }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HtmlRspackPluginOptions {
  /// emitted file name in output path
  #[serde(default = "default_filename")]
  pub filename: Vec<String>,
  /// template html file
  pub template: Option<String>,
  #[serde(skip)]
  pub template_fn: Option<TemplateRenderFn>,
  pub template_content: Option<String>,
  #[serde(skip)]
  pub template_parameters: TemplateParameters,
  /// `head`, `body`, `false`
  #[serde(default = "default_inject")]
  pub inject: HtmlInject,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, or `module`
  #[serde(default = "default_script_loading")]
  pub script_loading: HtmlScriptLoading,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub exclude_chunks: Option<Vec<String>>,
  pub chunks_sort_mode: HtmlChunkSortMode,

  #[serde(default)]
  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<FxHashMap<String, FxHashMap<String, String>>>,
  pub hash: Option<bool>,
  pub base: Option<HtmlRspackPluginBaseOptions>,
  /// uid is used to identify the plugin instance on javascript side
  pub uid: Option<u32>,
}

fn default_filename() -> Vec<String> {
  vec![String::from("index.html")]
}

fn default_script_loading() -> HtmlScriptLoading {
  HtmlScriptLoading::Defer
}

fn default_inject() -> HtmlInject {
  HtmlInject::Head
}

fn default_chunks_sort_mode() -> HtmlChunkSortMode {
  HtmlChunkSortMode::Auto
}

impl Default for HtmlRspackPluginOptions {
  fn default() -> HtmlRspackPluginOptions {
    HtmlRspackPluginOptions {
      filename: default_filename(),
      template: None,
      template_fn: None,
      template_content: None,
      template_parameters: TemplateParameters::Map(Default::default()),
      inject: default_inject(),
      public_path: None,
      script_loading: default_script_loading(),
      chunks: None,
      exclude_chunks: None,
      chunks_sort_mode: default_chunks_sort_mode(),
      minify: None,
      title: None,
      favicon: None,
      meta: None,
      hash: None,
      base: None,
      uid: None,
    }
  }
}

impl HtmlRspackPluginOptions {
  pub async fn get_public_path(&self, compilation: &Compilation, filename: &str) -> String {
    match &self.public_path {
      Some(p) => PublicPath::ensure_ends_with_slash(p.clone()),
      None => {
        compilation
          .options
          .output
          .public_path
          .render(compilation, filename)
          .await
      }
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
