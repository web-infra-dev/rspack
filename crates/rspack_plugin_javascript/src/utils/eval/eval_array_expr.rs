use rspack_util::SpanExt;
use swc_next_ecma_ast::{ArgumentData, ArrayExpression, GetSpan};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_array_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: ArrayExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let mut items = Vec::new();
  for element in expression
    .elements(ast)
    .iter()
    .map(|id| ast.get_node_in_sub_range(id))
  {
    let element = element?;
    let ArgumentData::Expr(element) = ast.argument_data(element) else {
      return None;
    };
    items.push(parser.evaluate_expression(element));
  }
  let span = expression.span(ast);
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  result.set_items(items);
  Some(result)
}
