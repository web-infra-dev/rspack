use rspack_core::{ConstDependency, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Ident, UnaryExpr, UnaryOp};

use super::JavascriptParserPlugin;
use crate::dependency::WebpackIsIncludedDependency;
use crate::visitors::JavascriptParser;

const WEBPACK_IS_INCLUDED: &str = "__webpack_is_included__";

fn is_webpack_is_included(ident: &Ident) -> bool {
  ident.sym.as_str() == WEBPACK_IS_INCLUDED
}

pub struct WebpackIsIncludedPlugin;

impl JavascriptParserPlugin for WebpackIsIncludedPlugin {
  fn call(&self, parser: &mut JavascriptParser<'_>, expr: &CallExpr, _name: &str) -> Option<bool> {
    let is_webpack_is_included = expr
      .callee
      .as_expr()
      .and_then(|expr| expr.as_ident())
      .map(is_webpack_is_included)
      .unwrap_or_default();
    if !is_webpack_is_included || expr.args.len() != 1 || expr.args[0].spread.is_some() {
      return None;
    }

    let request = parser.evaluate_expression(&expr.args[0].expr);
    if !request.is_string() {
      return None;
    }

    parser
      .dependencies
      .push(Box::new(WebpackIsIncludedDependency::new(
        expr.span().real_lo(),
        expr.span().hi().0 - 1,
        request.string().to_string(),
      )));

    Some(true)
  }

  fn r#typeof(&self, parser: &mut JavascriptParser<'_>, expr: &UnaryExpr) -> Option<bool> {
    assert!(expr.op == UnaryOp::TypeOf);
    let is_webpack_is_included = expr
      .arg
      .as_ident()
      .map(is_webpack_is_included)
      .unwrap_or_default();

    if !is_webpack_is_included {
      None
    } else {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().hi().0 - 1,
          "'function'".into(),
          None,
        )));
      Some(true)
    }
  }
}
