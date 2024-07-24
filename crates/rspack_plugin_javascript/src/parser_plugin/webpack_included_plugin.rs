use rspack_core::{ConstDependency, SpanExt};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::dependency::WebpackIsIncludedDependency;
use crate::visitors::JavascriptParser;

const WEBPACK_IS_INCLUDED: &str = "__webpack_is_included__";

pub struct WebpackIsIncludedPlugin;

impl JavascriptParserPlugin for WebpackIsIncludedPlugin {
  fn call(&self, parser: &mut JavascriptParser, expr: &CallExpr, name: &str) -> Option<bool> {
    if name != WEBPACK_IS_INCLUDED || expr.args.len() != 1 || expr.args[0].spread.is_some() {
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

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser<'_>,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    (for_name == WEBPACK_IS_INCLUDED).then(|| {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().hi().0 - 1,
          "'function'".into(),
          None,
        )));
      true
    })
  }
}
