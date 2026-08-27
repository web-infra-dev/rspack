use std::sync::Arc;

use rspack_core::{BuildMetaExportsType, ExportsArgument, ModuleArgument, ModuleType};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  AwaitExpression, CallExpression, ForOfStatement, GetSpan, Program, Span, UnaryExpression,
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ESMCompatibilityDependency,
  utils::eval::BasicEvaluatedExpression,
  visitors::{Identifier, JavascriptParser, create_traceable_error},
};

impl JavascriptParser<'_> {
  fn throw_top_level_await_error(&mut self, msg: String, span: Span) {
    self.add_error(
      create_traceable_error(
        "JavaScript parse error".into(),
        msg,
        self.source.to_string(),
        span.into(),
      )
      .into(),
    );
  }

  fn handle_top_level_await(&mut self, span: Span) {
    if self.is_esm {
      self.build_meta.set_has_top_level_await(true);
    } else {
      self.throw_top_level_await_error(
        "Top-level-await is only supported in ECMAScript Modules".into(),
        span,
      );
    }
  }
}

#[derive(Default)]
pub struct ESMDetectionParserPlugin;

// nonHarmonyIdentifiers
fn is_non_esm_identifier(name: &str) -> bool {
  name == "exports" || name == "define"
}

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ESMDetectionParserPlugin {
  fn program(&self, parser: &mut JavascriptParser<'p>, _program: Program) -> Option<bool> {
    let is_strict_esm = matches!(parser.module_type, ModuleType::JsEsm);
    let is_esm = parser.detect_esm_program(_program);

    if is_esm {
      parser.add_presentational_dependency(Arc::new(ESMCompatibilityDependency));
      parser.build_meta.set_esm(true);
      parser
        .build_meta
        .set_exports_type(BuildMetaExportsType::Namespace);
      parser.build_info.strict = true;
      parser.build_info.exports_argument = ExportsArgument::RspackExports;
    }

    if is_strict_esm {
      parser.build_meta.set_strict_esm_module(true);
      parser.build_info.module_argument = ModuleArgument::RspackModule;
    }

    None
  }

  fn top_level_await_expr(&self, parser: &mut JavascriptParser<'p>, expr: AwaitExpression) {
    let lo = expr.span(parser.ast.ast).real_lo();
    let hi = lo + AWAIT_LEN;
    let span = Span::new(lo, hi);
    parser.handle_top_level_await(span);
  }

  fn top_level_for_of_await_stmt(&self, parser: &mut JavascriptParser<'p>, stmt: ForOfStatement) {
    let offset = 4; // "for ".len();
    let lo = stmt.span(parser.ast.ast).real_lo() + offset;
    let hi = lo + AWAIT_LEN;
    let span = Span::new(lo, hi);
    parser.handle_top_level_await(span);
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: UnaryExpression,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'p>> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then(|| {
      let span = expr.span(parser.ast.ast);
      BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi())
    })
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: UnaryExpression,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then_some(true)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    _ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then_some(true)
  }

  fn call(
    &self,
    parser: &mut JavascriptParser<'p>,
    _expr: CallExpression,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && is_non_esm_identifier(for_name)).then_some(true)
  }
}

/// "await".len();
const AWAIT_LEN: u32 = 5;
