use std::sync::Arc;

use rspack_error::BatchErrors;
use rspack_util::swc::minify_file_comments;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
pub use swc_core::base::BoolOrDataConfig;
use swc_core::{
  atoms::Atom,
  base::{
    BoolOr,
    config::{IsModule, JsMinifyCommentOption, JsMinifyFormatOptions, SourceMapsConfig},
  },
  common::{
    BytePos, FileName, Mark,
    comments::{Comments, SingleThreadedComments},
    errors::HANDLER,
  },
  ecma::{
    ast::Ident,
    parser::{EsSyntax, Syntax},
    transforms::base::{
      fixer::{fixer, paren_remover},
      helpers::{self, Helpers},
      hygiene::hygiene,
      resolver,
    },
    visit::{Visit, VisitMutWith, noop_visit_type},
  },
};
pub use swc_ecma_minifier::option::{
  MangleOptions, MinifyOptions, TopLevelOptions,
  terser::{TerserCompressorOptions, TerserEcmaVersion},
};

use super::{
  JavaScriptCompiler, TransformOutput,
  stringify::{PrintOptions, SourceMapConfig},
};
use crate::error::with_rspack_error_handler;

impl JavaScriptCompiler {
  /// Minifies the given JavaScript source code.
  ///
  /// This method takes a filename, the source code to minify, minification options, and an optional function to operate on comments.
  /// It returns a `TransformOutput` containing the minified code and an optional source map.
  ///
  /// # Parameters
  ///
  /// - `filename`: The name of the file being minified.
  /// - `source`: The source code to minify.
  /// - `opts`: The options for minification.
  /// - `comments_op`: An optional function to operate on the comments in the source code.
  ///
  /// # Returns
  ///
  /// A `Result` containing a `TransformOutput` if the minification is successful, or a `BatchErrors` if an error occurs.
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
      with_rspack_error_handler("Minify Error".to_string(), self.cm.clone(), |handler| {
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
          if let Some(opts) = &mut min_opts.compress
            && opts.top_level.is_none()
          {
            opts.top_level = Some(TopLevelOptions { functions: true });
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
            import_attributes: true,
            ..Default::default()
          }),
          opts
            .module
            .map_or_else(|| IsModule::Unknown, IsModule::Bool),
          Some(&comments),
        )?;

        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let is_mangler_enabled = min_opts.mangle.is_some();

        let program = helpers::HELPERS.set(&Helpers::new(false), || {
          HANDLER.set(handler, || {
            let program = program
              .apply(&mut resolver(unresolved_mark, top_level_mark, false))
              .apply(&mut paren_remover(Some(&comments as &dyn Comments)));
            let mut program = swc_ecma_minifier::optimize(
              program,
              self.cm.clone(),
              Some(&comments),
              None,
              &min_opts,
              &swc_ecma_minifier::option::ExtraOptions {
                unresolved_mark,
                top_level_mark,
                mangle_name_cache: None,
              },
            );

            if !is_mangler_enabled {
              program.visit_mut_with(&mut hygiene())
            }
            program.apply(&mut fixer(Some(&comments as &dyn Comments)))
          })
        });

        if let Some(op) = comments_op {
          op(&comments);
        }

        minify_file_comments(
          &comments,
          &opts
            .format
            .comments
            .clone()
            .into_inner()
            .unwrap_or(BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments)),
          opts.format.preserve_annotations,
        );

        let print_options = PrintOptions {
          source_len: fm.byte_length(),
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
          preamble: &opts.format.preamble,
          ascii_only: opts.format.ascii_only,
          inline_script: opts.format.inline_script,
        };

        self.print(&program, print_options).map_err(|e| e.into())
      })
    })
  }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Represents the options for minifying JavaScript code.
pub struct JsMinifyOptions {
  #[serde(default = "true_as_default")]
  /// Indicates whether to minify the code.
  pub minify: bool,

  #[serde(default)]
  /// Configuration for compressing the code.
  pub compress: BoolOrDataConfig<TerserCompressorOptions>,

  #[serde(default)]
  /// Configuration for mangling names in the code.
  pub mangle: BoolOrDataConfig<MangleOptions>,

  #[serde(default)]
  /// Options for formatting the minified code.
  pub format: JsMinifyFormatOptions,

  #[serde(default)]
  /// The ECMAScript version to target.
  pub ecma: TerserEcmaVersion,

  #[serde(default, rename = "keep_classnames")]
  /// Indicates whether to keep class names unchanged.
  pub keep_class_names: bool,

  #[serde(default, rename = "keep_fnames")]
  /// Indicates whether to keep function names unchanged.
  pub keep_fn_names: bool,

  #[serde(default)]
  /// Indicates whether to wrap the code in a module.
  pub module: Option<bool>,

  #[serde(default)]
  /// Indicates whether to support Safari 10.
  pub safari10: bool,

  #[serde(default)]
  /// Indicates whether to scope the top level to the global object.
  pub toplevel: bool,

  #[serde(default)]
  /// Configuration for source maps.
  pub source_map: BoolOrDataConfig<TerserSourceMapKind>,

  #[serde(default)]
  /// The path where the minified output will be written.
  pub output_path: Option<String>,

  #[serde(default = "true_as_default")]
  /// Indicates whether to inline the source content in the source map.
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
