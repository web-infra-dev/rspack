use rspack_core::SpanExt;
use swc_core::ecma::ast::ArrayLit;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_array_expression(
  scanner: &mut JavascriptParser,
  expr: &ArrayLit,
) -> Option<BasicEvaluatedExpression> {
  let mut items = vec![];

  for elem in &expr.elems {
    if let Some(elem) = elem
      && elem.spread.is_none()
    {
      items.push(scanner.evaluate_expression(&elem.expr));
    } else {
      return None;
    }
  }

  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  res.set_items(items);
  Some(res)
}
