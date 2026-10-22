use rspack_util::SpanExt;
use swc_next_ecma_ast::{ConditionalExpression, GetSpan};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_cond_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: ConditionalExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let condition = parser.evaluate_expression(expression.test(ast));
  let mut result = if let Some(value) = condition.as_bool() {
    let mut selected = parser.evaluate_expression(if value {
      expression.consequent(ast)
    } else {
      expression.alternate(ast)
    });
    if condition.is_conditional() {
      selected.set_side_effects(true);
    }
    selected
  } else {
    let consequent = parser.evaluate_expression(expression.consequent(ast));
    let alternate = parser.evaluate_expression(expression.alternate(ast));
    let mut result = BasicEvaluatedExpression::new();
    if consequent.is_conditional() {
      result.set_options(consequent.into_options());
    } else {
      result.set_options(Some(vec![consequent]));
    }
    if alternate.is_conditional() {
      if let Some(options) = alternate.into_options() {
        result.add_options(options);
      }
    } else {
      result.add_options(vec![alternate]);
    }
    result
  };
  let span = expression.span(ast);
  result.set_range(span.real_lo(), span.real_hi());
  Some(result)
}
