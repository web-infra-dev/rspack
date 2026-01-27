use std::str::FromStr;

use napi::bindgen_prelude::{Either3, Promise};
use napi_derive::napi;
use rspack_napi::threadsafe_function::ThreadsafeFunction;
use rspack_plugin_html::config::{
  HtmlChunkSortMode, HtmlInject, HtmlRspackPluginBaseOptions, HtmlRspackPluginOptions,
  HtmlScriptLoading, TemplateParameterFn, TemplateParameters, TemplateRenderFn,
};
use rustc_hash::FxHashMap as HashMap;

pub type RawHtmlScriptLoading = String;
pub type RawHtmlInject = String;
pub type RawHtmlFilename = Vec<String>;
type RawChunkSortMode = String;

type RawTemplateRenderFn = ThreadsafeFunction<String, Promise<String>>;

type RawTemplateParameter =
  Either3<HashMap<String, String>, bool, ThreadsafeFunction<String, Promise<String>>>;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawHtmlRspackPluginOptions {
  /// emitted file name in output path
  #[napi(ts_type = "string[]")]
  pub filename: Option<RawHtmlFilename>,
  /// template html file
  pub template: Option<String>,
  #[napi(ts_type = "(data: string) => Promise<string>")]
  pub template_fn: Option<RawTemplateRenderFn>,
  pub template_content: Option<String>,
  #[napi(ts_type = "boolean | Record<string, any> | ((params: string) => Promise<string>)")]
  pub template_parameters: Option<RawTemplateParameter>,
  /// "head", "body" or "false"
  #[napi(ts_type = "\"head\" | \"body\" | \"false\"")]
  pub inject: RawHtmlInject,
  /// path or `auto`
  pub public_path: Option<String>,
  /// `blocking`, `defer`, `module` or `systemjs-module`
  #[napi(ts_type = "\"blocking\" | \"defer\" | \"module\" | \"systemjs-module\"")]
  pub script_loading: RawHtmlScriptLoading,

  /// entry_chunk_name (only entry chunks are supported)
  pub chunks: Option<Vec<String>>,
  pub exclude_chunks: Option<Vec<String>>,
  #[napi(ts_type = "\"auto\" | \"manual\"")]
  pub chunks_sort_mode: RawChunkSortMode,

  pub minify: Option<bool>,
  pub title: Option<String>,
  pub favicon: Option<String>,
  pub meta: Option<HashMap<String, HashMap<String, String>>>,
  pub hash: Option<bool>,
  pub base: Option<RawHtmlRspackPluginBaseOptions>,
  pub uid: Option<u32>,
}

impl From<RawHtmlRspackPluginOptions> for HtmlRspackPluginOptions {
  fn from(value: RawHtmlRspackPluginOptions) -> Self {
    let inject = HtmlInject::from_str(&value.inject).expect("Invalid inject value");

    let script_loading =
      HtmlScriptLoading::from_str(&value.script_loading).expect("Invalid script_loading value");

    let chunks_sort_mode =
      HtmlChunkSortMode::from_str(&value.chunks_sort_mode).expect("Invalid chunks_sort_mode value");

    HtmlRspackPluginOptions {
      filename: value
        .filename
        .unwrap_or_else(|| vec![String::from("index.html")]),
      template: value.template,
      template_fn: value.template_fn.map(|func| TemplateRenderFn {
        inner: Box::new(move |data| {
          let f = func.clone();
          Box::pin(async move { f.call_with_promise(data).await })
        }),
      }),
      template_content: value.template_content,
      template_parameters: match value.template_parameters {
        Some(parameters) => match parameters {
          Either3::A(data) => TemplateParameters::Map(data),
          Either3::B(enabled) => {
            if enabled {
              TemplateParameters::Map(Default::default())
            } else {
              TemplateParameters::Disabled
            }
          }
          Either3::C(func) => TemplateParameters::Function(TemplateParameterFn {
            inner: Box::new(move |data| {
              let f = func.clone();
              Box::pin(async move { f.call_with_promise(data).await })
            }),
          }),
        },
        None => TemplateParameters::Map(Default::default()),
      },
      inject,
      public_path: value.public_path,
      script_loading,
      chunks: value.chunks,
      exclude_chunks: value.exclude_chunks,
      chunks_sort_mode,
      minify: value.minify,
      title: value.title,
      favicon: value.favicon,
      meta: value.meta,
      hash: value.hash,
      base: value.base.map(|v| v.into()),
      uid: value.uid,
    }
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct RawHtmlRspackPluginBaseOptions {
  pub href: Option<String>,
  #[napi(ts_type = "\"_self\" | \"_blank\" | \"_parent\" | \"_top\"")]
  pub target: Option<String>,
}

impl From<RawHtmlRspackPluginBaseOptions> for HtmlRspackPluginBaseOptions {
  fn from(value: RawHtmlRspackPluginBaseOptions) -> Self {
    HtmlRspackPluginBaseOptions {
      href: value.href,
      target: value.target,
    }
  }
}
