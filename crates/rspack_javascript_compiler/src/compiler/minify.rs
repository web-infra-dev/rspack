use std::sync::Arc;

use rspack_error::BatchErrors;
use rspack_util::source_map::SourceMapKind;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
pub use swc_core::base::BoolOrDataConfig;
use swc_core::{
  atoms::Atom,
  base::{
    BoolOr,
    config::{IsModule, JsMinifyCommentOption, JsMinifyFormatOptions, SourceMapsConfig},
  },
  common::{
    BytePos, DUMMY_SP, FileName, Mark, SyntaxContext,
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
    errors::HANDLER,
  },
  ecma::{
    ast::{
      BlockStmt, CallExpr, Callee, Decl, EmptyStmt, Expr, Ident, Lit, ModuleItem, Pat,
      Program as SwcProgram, Stmt, Str, VarDecl, VarDeclKind, VarDeclarator,
    },
    parser::{EsSyntax, Syntax},
    transforms::base::{
      fixer::{fixer, paren_remover},
      hygiene::hygiene,
      resolver,
    },
    visit::{Visit, VisitMut, VisitMutWith, VisitWith, noop_visit_mut_type, noop_visit_type},
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

const MIN_DEDUPLICATED_STRING_SIZE: usize = 10_000;

#[derive(Default)]
struct LargeStringCollector {
  counts: FxHashMap<swc_core::atoms::Wtf8Atom, usize>,
  strings: Vec<Str>,
  used_symbols: FxHashSet<Atom>,
  has_direct_eval: bool,
}

impl Visit for LargeStringCollector {
  noop_visit_type!();

  fn visit_expr(&mut self, expr: &Expr) {
    if let Expr::Lit(Lit::Str(string)) = expr
      && string.value.as_bytes().len() >= MIN_DEDUPLICATED_STRING_SIZE
    {
      let count = self.counts.entry(string.value.clone()).or_default();
      if *count == 0 {
        self.strings.push(string.clone());
      }
      *count += 1;
    }
    expr.visit_children_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    self.used_symbols.insert(ident.sym.clone());
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(callee) = &call_expr.callee
      && let Expr::Ident(ident) = &**callee
      && ident.sym == "eval"
    {
      self.has_direct_eval = true;
    }
    call_expr.visit_children_with(self);
  }
}

struct LargeStringReplacer {
  replacements: FxHashMap<swc_core::atoms::Wtf8Atom, Ident>,
}

impl VisitMut for LargeStringReplacer {
  noop_visit_mut_type!();

  fn visit_mut_expr(&mut self, expr: &mut Expr) {
    if let Expr::Lit(Lit::Str(string)) = expr
      && let Some(ident) = self.replacements.get(&string.value)
    {
      *expr = Expr::Ident(Ident::new(
        ident.sym.clone(),
        string.span,
        SyntaxContext::empty(),
      ));
      return;
    }
    expr.visit_mut_children_with(self);
  }
}

/// Hoists repeated large strings only within one top-level expression. The
/// block keeps the generated binding out of the global lexical scope while
/// allowing nested module factories to capture it.
fn deduplicate_large_strings_in_statement(statement: &mut Stmt) {
  let Stmt::Expr(expr_statement) = statement else {
    return;
  };

  let mut collector = LargeStringCollector::default();
  expr_statement.expr.visit_with(&mut collector);
  if collector.has_direct_eval {
    return;
  }

  let mut replacement_index = 0;
  let mut replacements = Vec::new();
  for string in collector.strings {
    if collector.counts.get(&string.value).copied().unwrap_or(0) < 2 {
      continue;
    }

    let ident = loop {
      let symbol = Atom::from(format!("__rspack_string_{replacement_index}"));
      replacement_index += 1;
      if !collector.used_symbols.contains(&symbol) {
        break Ident::new(symbol, DUMMY_SP, SyntaxContext::empty());
      }
    };
    replacements.push((string, ident));
  }

  if replacements.is_empty() {
    return;
  }

  let mut replacer = LargeStringReplacer {
    replacements: replacements
      .iter()
      .map(|(string, ident)| (string.value.clone(), ident.clone()))
      .collect(),
  };
  expr_statement.expr.visit_mut_with(&mut replacer);

  let declaration = Stmt::Decl(Decl::Var(Box::new(VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Const,
    decls: replacements
      .into_iter()
      .map(|(string, ident)| VarDeclarator {
        span: DUMMY_SP,
        name: Pat::Ident(ident.into()),
        init: Some(Box::new(Expr::Lit(Lit::Str(string)))),
        definite: false,
      })
      .collect(),
    ..Default::default()
  })));
  let original = std::mem::replace(statement, Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
  *statement = Stmt::Block(BlockStmt {
    span: DUMMY_SP,
    stmts: vec![declaration, original],
    ..Default::default()
  });
}

fn deduplicate_large_strings(program: &mut SwcProgram) {
  match program {
    SwcProgram::Script(script) => {
      for statement in &mut script.body {
        deduplicate_large_strings_in_statement(statement);
      }
    }
    SwcProgram::Module(module) => {
      for item in &mut module.body {
        if let ModuleItem::Stmt(statement) = item {
          deduplicate_large_strings_in_statement(statement);
        }
      }
    }
  }
}

/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/e6fc5327b1a309eae840fe1ec3a2367adab37430/crates/swc_compiler_base/src/lib.rs#L342
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
pub(super) fn minify_file_comments(
  comments: &SingleThreadedComments,
  preserve_comments: &BoolOr<JsMinifyCommentOption>,
  preserve_annotations: bool,
) {
  match preserve_comments {
    BoolOr::Bool(true) | BoolOr::Data(JsMinifyCommentOption::PreserveAllComments) => {}

    BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments) => {
      let preserve_excl = |_: &BytePos, vc: &mut std::vec::Vec<Comment>| -> bool {
        // Preserve license comments.
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L175-L181
        vc.retain(|c: &Comment| {
          c.text.contains("@lic")
            || c.text.contains("@preserve")
            || c.text.contains("@copyright")
            || c.text.contains("@cc_on")
            || (preserve_annotations
              && (c.text.contains("__PURE__")
                || c.text.contains("__INLINE__")
                || c.text.contains("__NOINLINE__")
                || c.text.contains("@vite-ignore")))
            || (c.kind == CommentKind::Block && c.text.starts_with('!'))
        });
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();

      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }

    BoolOr::Bool(false) => {
      let (mut l, mut t) = comments.borrow_all_mut();
      l.clear();
      t.clear();
    }
    BoolOr::Data(JsMinifyCommentOption::PreserveRegexComments { regex }) => {
      let preserve_excl = |_: &BytePos, vc: &mut std::vec::Vec<Comment>| -> bool {
        // Preserve comments that match the regex
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L286
        vc.retain(|c: &Comment| regex.find(&c.text).is_some());
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();
      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }
  }
}

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
          .expect("should have source map config");
        let source_map_kind = SourceMapKind::from_enabled(source_map.enabled())
          .with_sources_content(opts.inline_sources_content);

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

        let program = HANDLER.set(handler, || {
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
          deduplicate_large_strings(&mut program);
          program.apply(&mut fixer(Some(&comments as &dyn Comments)))
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
            source_map_kind,
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

#[cfg(test)]
mod tests {
  use super::*;

  fn minify_without_compression(source: String) -> String {
    JavaScriptCompiler::new()
      .minify(
        FileName::Custom("test.js".into()),
        source,
        JsMinifyOptions {
          minify: true,
          compress: BoolOrDataConfig::from_bool(false),
          mangle: BoolOrDataConfig::from_bool(false),
          ..Default::default()
        },
        None::<fn(&SingleThreadedComments)>,
      )
      .expect("minification should succeed")
      .code
  }

  #[test]
  fn deduplicates_large_strings_in_one_top_level_expression() {
    let large_string = "x".repeat(MIN_DEDUPLICATED_STRING_SIZE);
    let output = minify_without_compression(format!(
      "globalThis.values = [{large_string:?}, {large_string:?}]"
    ));

    assert_eq!(output.matches(&large_string).count(), 1);
    assert!(output.contains("const __rspack_string_0="));
  }

  #[test]
  fn preserves_large_strings_across_top_level_expressions() {
    let large_string = "x".repeat(MIN_DEDUPLICATED_STRING_SIZE);
    let output = minify_without_compression(format!(
      "globalThis.first = {large_string:?}; globalThis.second = {large_string:?}"
    ));

    assert_eq!(output.matches(&large_string).count(), 2);
    assert!(!output.contains("__rspack_string_"));
  }

  #[test]
  fn skips_statements_containing_direct_eval() {
    let large_string = "x".repeat(MIN_DEDUPLICATED_STRING_SIZE);
    let output = minify_without_compression(format!(
      "globalThis.values = [eval('0'), {large_string:?}, {large_string:?}]"
    ));

    assert_eq!(output.matches(&large_string).count(), 2);
    assert!(!output.contains("__rspack_string_"));
  }
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
