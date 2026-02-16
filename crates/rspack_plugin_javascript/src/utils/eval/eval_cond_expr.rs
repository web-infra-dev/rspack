use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{CondExpr, Spanned};

use super::BasicEvaluatedExpression;
use crate::visitors::JavascriptParser;

#[inline]
pub fn eval_cond_expression(
  scanner: &mut JavascriptParser,
  cond: CondExpr,
) -> Option<BasicEvaluatedExpression> {
  let condition = scanner.evaluate_expression(cond.test(&scanner.ast));
  let condition_value = condition.as_bool();
  let mut res;
  if let Some(bool) = condition_value {
    if bool {
      res = scanner.evaluate_expression(cond.cons(&scanner.ast))
    } else {
      res = scanner.evaluate_expression(cond.alt(&scanner.ast))
    };
    if condition.is_conditional() {
      res.set_side_effects(true)
    }
  } else {
    let cons = scanner.evaluate_expression(cond.cons(&scanner.ast));
    let alt = scanner.evaluate_expression(cond.alt(&scanner.ast));
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
  res.set_range(
    cond.span(&scanner.ast).real_lo(),
    cond.span(&scanner.ast).real_hi(),
  );
  Some(res)
}
