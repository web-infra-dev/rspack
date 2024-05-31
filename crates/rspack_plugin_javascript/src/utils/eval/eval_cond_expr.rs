use swc_core::ecma::ast::CondExpr;

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_cond_expression(
  scanner: &mut JavascriptParser,
  cond: &CondExpr,
) -> Option<BasicEvaluatedExpression> {
  let condition = scanner.evaluate_expression(&cond.test);
  let condition_value = condition.as_bool();
  let mut res;
  if let Some(bool) = condition_value {
    if bool {
      res = scanner.evaluate_expression(&cond.cons)
    } else {
      res = scanner.evaluate_expression(&cond.alt)
    };
    if condition.is_conditional() {
      res.set_side_effects(true)
    }
  } else {
    let cons = scanner.evaluate_expression(&cond.cons);
    let alt = scanner.evaluate_expression(&cond.alt);
    res = BasicEvaluatedExpression::new();
    if cons.is_conditional() {
      res.set_options(cons.options)
    } else {
      res.set_options(Some(vec![cons]))
    }
    if alt.is_conditional() {
      if let Some(options) = alt.options {
        res.add_options(options)
      }
    } else {
      res.add_options(vec![alt])
    }
  }
  res.set_range(cond.span.lo.0, cond.span.hi.0);
  Some(res)
}
