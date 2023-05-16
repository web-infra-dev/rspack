#![feature(let_chains)]
#![feature(box_patterns)]

pub(crate) mod dependency;
mod plugin;
pub use plugin::*;
mod ast;
pub(crate) mod parser_and_generator;
pub mod runtime;
pub mod utils;
pub mod visitors;

use swc_config::config_types::BoolOrDataConfig;
use swc_ecma_minifier::option::{
  terser::{TerserCompressorOptions, TerserEcmaVersion},
  MangleOptions,
};

pub use crate::plugin::infer_async_modules_plugin::InferAsyncModulesPlugin;

#[derive(Debug)]
pub struct TransformOutput {
  pub code: String,
  pub map: Option<String>,
}

#[derive(Debug)]
pub enum SourceMapsConfig {
  Bool(bool),
  Str(String),
}

impl SourceMapsConfig {
  pub fn enabled(&self) -> bool {
    match *self {
      SourceMapsConfig::Bool(b) => b,
      SourceMapsConfig::Str(ref s) => {
        assert_eq!(s, "inline", "Source map must be true, false or inline");
        true
      }
    }
  }
}

#[derive(Debug)]
pub enum IsModule {
  Bool(bool),
  Unknown,
}

#[derive(Debug, Clone)]
pub enum JsMinifyCommentOption {
  PreserveSomeComments,
  PreserveAllComments,
}

#[derive(Debug, Clone, Default)]
pub struct JsMinifyOptions {
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,
  pub mangle: BoolOrDataConfig<MangleOptions>,
  pub format: JsMinifyFormatOptions,
  pub ecma: TerserEcmaVersion,
  pub keep_classnames: bool,
  pub keep_fnames: bool,
  pub module: bool,
  pub safari10: bool,
  pub toplevel: bool,
  pub source_map: BoolOrDataConfig<TerserSourceMapOption>,
  pub output_path: Option<String>,
  pub inline_sources_content: bool,
  pub emit_source_map_columns: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TerserSourceMapOption {
  pub filename: Option<String>,
  pub url: Option<String>,
  pub root: Option<String>,
  pub content: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JsMinifyFormatOptions {
  pub ascii_only: bool,
  pub beautify: bool,
  pub braces: bool,
  pub comments: BoolOrDataConfig<JsMinifyCommentOption>,
  pub ecma: usize,
  pub indent_level: Option<usize>,
  pub indent_start: bool,
  pub inline_script: bool,
  pub keep_numbers: bool,
  pub keep_quoted_props: bool,
  pub max_line_len: usize,
  pub preamble: String,
  pub quote_keys: bool,
  pub quote_style: usize,
  pub preserve_annotations: bool,
  pub safari10: bool,
  pub semicolons: bool,
  pub shebang: bool,
  pub webkit: bool,
  pub wrap_iife: bool,
  pub wrap_func_args: bool,
}
