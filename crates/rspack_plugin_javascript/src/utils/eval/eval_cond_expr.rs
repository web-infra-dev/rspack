use swc_core::ecma::ast::CondExpr;

use super::BasicEvaluatedExpression;
use crate::{parser_plugin::JavaScriptParserPluginDrive, visitors::JavascriptParser};

pub fn eval_cond_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  cond: &'ast CondExpr,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let condition = scanner.evaluate_expression(&cond.test, plugin_drive);
  let condition_value = condition.as_bool();
  let mut res;
  if let Some(bool) = condition_value {
    if bool {
      res = scanner.evaluate_expression(&cond.cons, plugin_drive);
    } else {
      res = scanner.evaluate_expression(&cond.alt, plugin_drive);
    };
    if condition.is_conditional() {
      res.set_side_effects(true)
    }
  } else {
    let cons = scanner.evaluate_expression(&cond.cons, plugin_drive);
    let alt = scanner.evaluate_expression(&cond.alt, plugin_drive);
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
