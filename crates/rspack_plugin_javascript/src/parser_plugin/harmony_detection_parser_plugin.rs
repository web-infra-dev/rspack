use std::ops::Add;

use rspack_core::{BuildMetaExportsType, ExportsArgument, ModuleArgument, ModuleType, SpanExt};
use swc_core::common::source_map::Pos;
use swc_core::common::{BytePos, Span, Spanned};
use swc_core::ecma::ast::{Ident, ModuleItem, Program, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::dependency::HarmonyCompatibilityDependency;
use crate::utils::eval::BasicEvaluatedExpression;
use crate::visitors::{create_traceable_error, JavascriptParser};

impl<'parser> JavascriptParser<'parser> {
  fn throw_top_level_await_error(&mut self, msg: String, span: Span) {
    self.errors.push(Box::new(create_traceable_error(
      "JavaScript parsing error".into(),
      msg,
      self.source_file,
      span.into(),
    )));
  }

  fn handle_top_level_await(&mut self, allow_top_level: bool, span: Span) {
    if !allow_top_level {
      self.throw_top_level_await_error("The top-level-await experiment is not enabled (set experiments.topLevelAwait: true to enabled it)".into(), span);
    } else if self.is_esm {
      self.build_meta.has_top_level_await = true;
    } else {
      self.throw_top_level_await_error(
        "Top-level-await is only supported in EcmaScript Modules".into(),
        span,
      );
    }
  }
}

pub struct HarmonyDetectionParserPlugin {
  top_level_await: bool,
}

impl HarmonyDetectionParserPlugin {
  pub fn new(top_level_await: bool) -> Self {
    Self { top_level_await }
  }
}

// Port from https://github.com/webpack/webpack/blob/main/lib/dependencies/HarmonyDetectionParserPlugin.js
impl JavascriptParserPlugin for HarmonyDetectionParserPlugin {
  fn program(&self, parser: &mut JavascriptParser, ast: &Program) -> Option<bool> {
    let is_strict_harmony = matches!(parser.module_type, ModuleType::JsEsm);
    let is_harmony = is_strict_harmony
      || matches!(ast, Program::Module(module) if module.body.iter().any(|s| matches!(s, ModuleItem::ModuleDecl(_))));

    if is_harmony {
      parser
        .presentational_dependencies
        .push(Box::new(HarmonyCompatibilityDependency));
      parser.build_meta.esm = true;
      parser.build_meta.exports_type = BuildMetaExportsType::Namespace;
      parser.build_info.strict = true;
      parser.build_meta.exports_argument = ExportsArgument::WebpackExports;
    }

    if is_strict_harmony {
      parser.build_meta.strict_harmony_module = true;
      parser.build_meta.module_argument = ModuleArgument::WebpackModule;
    }

    None
  }

  fn top_level_await_expr(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::AwaitExpr,
  ) {
    let lo = expr.span_lo();
    let hi = lo.add(BytePos::from_u32(AWAIT_LEN));
    let span = Span::new(lo, hi, expr.span.ctxt);
    parser.handle_top_level_await(self.top_level_await, span);
  }

  fn top_level_for_of_await_stmt(
    &self,
    parser: &mut JavascriptParser,
    stmt: &swc_core::ecma::ast::ForOfStmt,
  ) {
    let offset = 4; // "for ".len();
    let lo = stmt.span_lo().add(BytePos::from_u32(offset));
    let hi = lo.add(BytePos::from_u32(AWAIT_LEN));
    let span = Span::new(lo, hi, stmt.span.ctxt);
    parser.handle_top_level_await(self.top_level_await, span);
  }

  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    (parser.is_esm && for_name == "exports")
      .then(|| BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0))
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    (parser.is_esm && for_name == "exports").then_some(true)
  }
}

/// "await".len();
const AWAIT_LEN: u32 = 5;
