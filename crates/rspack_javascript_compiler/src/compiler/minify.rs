use std::sync::Arc;

use rspack_error::{BatchErrors, DiagnosticKind};
use rspack_util::swc::minify_file_comments;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
pub use swc_core::base::BoolOrDataConfig;
use swc_core::{
  atoms::Atom,
  base::{
    config::{IsModule, JsMinifyCommentOption, JsMinifyFormatOptions, SourceMapsConfig},
    BoolOr,
  },
  common::{comments::SingleThreadedComments, BytePos, FileName},
  ecma::{
    ast::Ident,
    parser::{EsSyntax, Syntax},
    visit::{noop_visit_type, Visit},
  },
};
pub use swc_ecma_minifier::option::{
  terser::{TerserCompressorOptions, TerserEcmaVersion},
  MangleOptions, MinifyOptions, TopLevelOptions,
};

use super::{
  stringify::{PrintOptions, SourceMapConfig},
  JavaScriptCompiler, TransformOutput,
};
use crate::error::with_rspack_error_handler;

impl JavaScriptCompiler {
  pub fn minify<S: Into<String>, F>(
    &self,
    filename: FileName,
    source: S,
    opts: JsMinifyOptions,
    comments_op: Option<F>,
  ) -> Result<TransformOutput, BatchErrors>
  where
    F: for<'a> FnOnce(&'a SingleThreadedComments),
  {
    self.run(|| -> Result<TransformOutput, BatchErrors> {
      with_rspack_error_handler(
        "Minify Error".to_string(),
        DiagnosticKind::JavaScript,
        self.cm.clone(),
        |_handler| {
          let fm = self.cm.new_source_file(Arc::new(filename), source.into());

          let source_map = opts
            .source_map
            .as_ref()
            .map(|_| SourceMapsConfig::Bool(true))
            .unwrap_as_option(|v| {
              Some(match v {
                Some(true) => SourceMapsConfig::Bool(true),
                _ => SourceMapsConfig::Bool(false),
              })
            })
            .expect("TODO:");

          let mut min_opts = MinifyOptions {
            compress: opts
              .compress
              .clone()
              .unwrap_as_option(|default| match default {
                Some(true) | None => Some(Default::default()),
                _ => None,
              })
              .map(|v| v.into_config(self.cm.clone())),
            mangle: opts
              .mangle
              .clone()
              .unwrap_as_option(|default| match default {
                Some(true) | None => Some(Default::default()),
                _ => None,
              }),
            ..Default::default()
          };

          // top_level defaults to true if module is true

          // https://github.com/swc-project/swc/issues/2254
          if opts.module.unwrap_or(false) {
            if let Some(opts) = &mut min_opts.compress {
              if opts.top_level.is_none() {
                opts.top_level = Some(TopLevelOptions { functions: true });
              }
            }

            if let Some(opts) = &mut min_opts.mangle {
              opts.top_level = Some(true);
            }
          }

          let comments = SingleThreadedComments::default();

          let target = opts.ecma.clone().into();
          let program = self.parse_js(
            fm.clone(),
            target,
            Syntax::Es(EsSyntax {
              jsx: true,
              decorators: true,
              decorators_before_export: true,
              ..Default::default()
            }),
            opts
              .module
              .map_or_else(|| IsModule::Unknown, IsModule::Bool),
            Some(&comments),
          )?;

          if let Some(op) = comments_op {
            op(&comments);
          }

          minify_file_comments(
            &comments,
            opts
              .format
              .comments
              .clone()
              .into_inner()
              .unwrap_or(BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments)),
            opts.format.preserve_annotations,
          );

          let print_options = PrintOptions {
            source_map: self.cm.clone(),
            target,
            source_map_config: SourceMapConfig {
              enable: source_map.enabled(),
              inline_sources_content: opts.inline_sources_content,
              emit_columns: true,
              names: Default::default(),
            },
            input_source_map: None,
            minify: opts.minify,
            comments: Some(&comments),
            format: &opts.format,
          };

          self.print(&program, print_options).map_err(|e| e.into())
        },
      )
    })
  }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMinifyOptions {
  #[serde(default = "true_as_default")]
  pub minify: bool,
  #[serde(default)]
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,
  #[serde(default)]
  pub mangle: BoolOrDataConfig<MangleOptions>,
  #[serde(default)]
  pub format: JsMinifyFormatOptions,
  #[serde(default)]
  pub ecma: TerserEcmaVersion,
  #[serde(default, rename = "keep_classnames")]
  pub keep_class_names: bool,
  #[serde(default, rename = "keep_fnames")]
  pub keep_fn_names: bool,
  #[serde(default)]
  pub module: Option<bool>,
  #[serde(default)]
  pub safari10: bool,
  #[serde(default)]
  pub toplevel: bool,
  #[serde(default)]
  pub source_map: BoolOrDataConfig<TerserSourceMapKind>,
  #[serde(default)]
  pub output_path: Option<String>,
  #[serde(default = "true_as_default")]
  pub inline_sources_content: bool,
}

const fn true_as_default() -> bool {
  true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerserSourceMapKind {
  pub filename: Option<String>,
  pub url: Option<String>,
  pub root: Option<String>,
  pub content: Option<String>,
}

pub struct IdentCollector {
  pub names: FxHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
