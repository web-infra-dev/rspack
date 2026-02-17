use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{ArrayLit, GetSpan};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_array_expression(
  scanner: &mut JavascriptParser,
  expr: ArrayLit,
) -> Option<BasicEvaluatedExpression> {
  let mut items = vec![];

  for elem in expr.elems(&scanner.ast).iter() {
    let elem = scanner.ast.get_node_in_sub_range(elem);
    if let Some(elem) = elem
      && elem.spread(&scanner.ast).is_none()
    {
      items.push(scanner.evaluate_expression(elem.expr(&scanner.ast)));
    } else {
      return None;
    }
  }

  let mut res = BasicEvaluatedExpression::with_range(
    expr.span(&scanner.ast).real_lo(),
    expr.span(&scanner.ast).real_hi(),
  );
  res.set_items(items);
  Some(res)
}
