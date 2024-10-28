use rspack_core::{ConstDependency, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast::CallExpr;

use super::JavascriptParserPlugin;
use crate::visitors::{expr_matcher, JavascriptParser};

pub struct RequireJsStuffPlugin {}

impl JavascriptParserPlugin for RequireJsStuffPlugin {
  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == "require.config" || for_name == "requirejs.config" {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          call_expr.span().real_lo(),
          call_expr.span().real_hi(),
          "undefined".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if expr_matcher::is_require_version(expr) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "\"0.0.0\"".into(),
          None,
        )));
      Some(true)
    } else if expr_matcher::is_require_onerror(expr) {
      // TODO: add RuntimeGlobals.uncaughtErrorHandler
      None
    } else {
      None
    }
  }
}
